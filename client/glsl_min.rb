#!/usr/bin/env ruby
### Modified from https://github.com/flowtsohg/glsl-minifier

# Copyright (c) 2013 Chananya Freiman (aka GhostWolf)

# Used to generate all the permutations of vector fields of length 2-4 (xyzw, rgba, stpq)
# E.g., xx xy xz xw yx yy yz yw ...
def gen_sizzle_permutations()
  perms = []

  (2..4).each { |k|
    perms += ["x", "y", "z", "w"].repeated_permutation(k).to_a().map! { |v| v.join("") }
    perms += ["r", "g", "b", "a"].repeated_permutation(k).to_a().map! { |v| v.join("") }
    perms += ["s", "t", "p", "q"].repeated_permutation(k).to_a().map! { |v| v.join("") }
  }

  return perms
end

# Rename language keywords and types using #defines where it will reduce the overall size
def add_defines(data, language_words)
  defines = []

  language_words.each { |v|
    uses = data.scan(/\b#{v[0]}\b/).length
    usage = uses * v[0].length

    if usage > 0
      define = "#define #{v[1]} #{v[0]}\n"
      define_usage = define.length + uses * v[1].length

      if define_usage < usage
        defines.push("#define #{v[1]} #{v[0]}")

        data.gsub!(/\b#{v[0]}\b/, v[1])
      end
    end
  }

  return "\n" + defines.join("\n") + "\n" + data
end

def preprocess_defines(defines)
  rewrites = {}

  # Get inline values for simple number equation #defines
  # E.g.: 5 => 5, 5*2 => 10
  defines.each { |v|
    begin
      n = eval(v[2]).to_s()
      rewrites[v[1]] = n
      v[3] = n
    rescue
    end
  }

  # Get inline values for #define equations that have the previously inlined #defines in their values
  # E.g.: N/2 => 5/2 => 2.5, assuming N was inlined as 5
  defines.each { |v|
    if not v[3]
      begin
        s = v[2]
        rewrites.each { |k, n|
          s.gsub!(/\b#{k}\b/, n)
        }

        n = eval(s).to_s()
        v[3] = n
      rescue
      end
    end
  }
end

def inline_defines(defines, data)
  # First pass removes the inlined #define lines
  defines.each { |v|
    if v[3]
      data.sub!(v[0], "")
    end
  }

  # Second pass inlines the values
  defines.each { |v|
    if v[3]
      data.gsub!(/\b#{v[1]}\b/, v[3])
    end
  }
end

def rewrite_numbers(data)
  # Convert hexadecimal numbers to decimal numbers
  data.gsub!(/0x[0-9a-fA-F]+/) { |n|
    n.to_i(16)
  }

  # Remove useless zeroes
  data.gsub!(/\b\d*\.?\d+\b/) { |n|
    if n["."]
      n.to_f().to_s()
    else
      n.to_i().to_s()
    end
  }

  # Remove useless zeroes
  data.gsub!(/[0]+(\.\d+)/, "\\1")
  data.gsub!(/(\d+\.)[0]+/, "\\1")

  # Change integers to exponent representation if it's shorter
  data.gsub!(/(\d+?)(0+)/) {
    n = $1
    e = $2

    if e.size > 2
      "#{n}e#{(e.size - 1)}"
    else
      n + e
    end
  }
end

# Removes comments
def remove_comments(source)
  source.gsub!(/\/\/.*/, "")
  source.gsub!(/\/\*.*?\*\//m, "")
end

def parse_structs(data, datatypes)
  data.split(/(struct\s+(?:\w+)\s*{.*?}\s*;)/m)
end

def rename_struct_members(chunk, map, datatypes)
  map.each { |v|
    chunk.sub!(/\b#{v[0]}\b/, map[v[0]])
  }
end

# Rename the members of all structs
def rename_members(data, map, datatypes)
  source = ""

  parse_structs(data, datatypes).each_with_index { |chunk, i|
    if i % 2 != 0
      # Rename the members
      rename_struct_members(chunk, map, datatypes)

      tokens = chunk.split(/(struct \w+{)(.*?)(};)/m)

      source += tokens[1]

      # All global variables in global scope can be combined, unless they exist in #ifdefs
      outer = tokens[2].gsub(/(#if.*?#endif)/m, "")

      source += group_list(outer.scan(/()(#{datatypes})\s+(\w+)(.*?);/))

      tokens[2].split(/(#ifdef.*?#endif)/m).each { |chunk|
        # Do the same thing inside #ifdefs
        if chunk.start_with?("#if")
          tokens = chunk.split(/(#ifdef.*?\n)(.*?)(#endif)/m)

          source += tokens[1]
          source += group_list(tokens[2].scan(/()(#{datatypes})\s+(\w+)(.*?);/))
          source += tokens[2].gsub(/(#{datatypes})\s+(\w+)(.*?);/, "")
          source += tokens[3]
          source += "\n"
        else
          source += chunk.gsub(/(#{datatypes})\s+(\w+)(.*?);/, "")
        end
      }

      source += "};"
    else
      source += chunk
    end
  }

  map.each { |v|
    source.gsub!(/\.\b#{v[0]}\b/, ".#{map[v[0]]}")
  }

  return source
end

def get_member_names(data, datatypes)
  data.scan(/(struct\s+(?:\w+)\s*{.*?}\s*;)/m).collect { |struct|
    struct[0].scan(/(?:#{datatypes})\s+(\w+)\s*;/)
  }.flatten(1)
end

# Split the source into function/other chunks
def parse_functions(source, datatypes)
  pass = source.split(/((?:#{datatypes})\s+\w+\s*\(.*?\))/)
  chunks = [[pass[0]]]

  (1...pass.size).step(2) { |i|
    head = pass[i].split(/\(/)
    body_with_extra = pass[i + 1]

    start = body_with_extra.index("{")
    index = start + 1
    level = 1

    while level > 0 and index < body_with_extra.length do
      char = body_with_extra[index]

      if char == "}"
        level -= 1
      elsif char == "{"
        level += 1
      end

      index += 1
    end

    body = body_with_extra[0...index]
    extra = body_with_extra[index..body_with_extra.size]

    chunks +=  [[head[0], "(" + head[1], body], [extra]]
  }

  return chunks
end

# Rename function arguments and local variables.
def rename_function_locals(data, datatypes)
  names = [*("a".."z"), *("aa".."zz")]
  arguments = []
  locals = []

  # Grab all the argument names
  data[1].scan(/(?:in\s+|out\s+)?(?:#{datatypes})\s+(\w+)/).each { |argument|
    arguments += argument
  }

  # Short names must always come before longer names
  arguments.sort!()

  data[2].scan(/(#{datatypes}) (.*?);/m).each { |local_list|
    local_list[1].split("=")[0].split(",").each { |local|
      locals += [local.strip()]
    }
  }

  # Short names must always come before longer names
  locals.sort!()

  # Rename function arguments
  arguments.each { |argument|
    name = names.shift()
    reg = /\b#{argument}\b/

    data[1].sub!(reg, name)
    data[2].gsub!(reg, name)
  }

  # Rename function locals
  locals.each { |local|
    data[2].gsub!(/\b#{local.strip()}\b/, names.shift())
  }
end

# Removes useless whitespace from the source
def remove_whitespace(oldsource)
  need_newline = false
  source = ""

  oldsource = oldsource.sub(/^\n+/, "")

  oldsource.each_line { |line|
    line = line.strip().gsub(/\s{2,}|\t/, " ")

    if line[0] == "#"
      if need_newline
        source += "\n"
      end

      source += line + "\n"
      need_newline = false
    else
      source += line.sub("\n", "").gsub(/\s*({|}|=|\*|,|\+|\/|>|<|&|\||\[|\]|\(|\)|\-|!|;)\s*/, "\\1")
      need_newline = true
    end
  }

  return source.gsub(/\n+/, "\n")
end

def get_used_functions_in_function(used_functions, main_function, function_chunks)
  function_chunks.each { |f|
    match = main_function[2][/\b#{f[0]}\b/]

    if match
      used_functions[match] = 1

      get_used_functions_in_function(used_functions, function_chunks[match], function_chunks)
    end
  }
end

# Removes dead functions
def remove_dead_functions(shaders, datatypes)
  functions = []
  used_functions = {"main" => 1}
  function_chunks = {}
  main_chunks = []
  shaders.each { |shader|
    parse_functions(shader, datatypes).each_with_index { |chunk, i|
      if i % 2 != 0
        name = chunk[0].split(/\s+/)[1]

        if name != "main"
          function_chunks[name] = chunk
        else
          main_chunks.push(chunk)
        end
      end
    }
  }

  main_chunks.each { |main_function|
    get_used_functions_in_function(used_functions, main_function, function_chunks)
  }

  shaders.map! { |shader|
    source = ""

    parse_functions(shader, datatypes).each_with_index { |chunk, i|
      if i % 2 != 0
        if used_functions[chunk[0].split(/\s+/)[1]]
          source += chunk.join("")
        end
      else
        source += chunk.join("")
      end
    }

    source
  }
end

# Renames function arguments and local variables of all functions
def rename_locals(oldsource, datatypes)
  source = ""

  parse_functions(oldsource, datatypes).each_with_index { |chunk, i|
    rename_function_locals(chunk, datatypes) if i % 2 != 0

    source += chunk.join("")
  }

  return source
end

# Gets all the user defined type names
def get_struct_names(data)
  data.scan(/struct\s+(\w+)/)
end

# Gets all the defines and their values
def get_defines(data)
  data.scan(/(#define\s+(\w+)\s+([^\n]+))/).map { |match|
    [match[0], match[1], match[2].strip()]
  }
end

# Gets the names of all functions
def get_function_names(data, datatypes)
  data.scan(/(?:#{datatypes})\s+(\w+)\s*\(/)
end

# Gets the names of all variables with the given qualifier
def get_variable_names(data, qualifier, datatypes)
  data.scan(/#{qualifier}\s+(?:#{datatypes})\s+(\w+)\s*;/)
end

# Generate a old name to new name mapping and sort the names alphabetically
def gen_map(data, names, rewrite)
  # Select new short names for all the functions
  map = data.uniq().map { |v|
    if rewrite and v[0] != "main"
      [v[0], names.shift()]
    else
      [v[0], v[0]]
    end
  }

  # Short names must always come before longer names
  map.sort! { |a, b|
    a[0] <=> b[0]
  }

  map
end

# Generate a old name to new name mapping and sort the names alphabetically
def gen_map_map(data, names, rewrite)
  # Select new short names for all the functions
  map = data.uniq().map { |v|
    if rewrite
      [v[0], names.shift()]
    else
      [v[0], v[0]]
    end
  }

  # Short names must always come before longer names
  map.sort! { |a, b|
    a[0] <=> b[0]
  }

  mapmap = {}

  map.each { |v|
    mapmap[v[0]] = v[1]
  }

  mapmap
end

# Rewrite tokens based on a map generated by gen_map
def rewrite_map(map, data)
  map.each { |v|
    data.gsub!(/\b#{v[0]}\b/, v[1])
  }
end

def group_list(list)
  map = {}
  data = ""

  list.each { |v|
    if not map[v[0]]
      map[v[0]] = {}
    end

    if not map[v[0]][v[1]]
      map[v[0]][v[1]] = []
    end

    map[v[0]][v[1]].push([v[2], v[3]])
  }

  map.each { |qualifier, v|
    v.each { |datatype, v|
      data += "#{qualifier} #{datatype} #{v.collect { |v| "#{v[0]}#{v[1]}"}.join(",")};"
    }
  }

  return data
end

def group_globals(data, datatypes)
  source = ""

  # All global variables in global scope can be combined, unless they exist in #ifdefs
  outer = data.gsub(/(#if.*?#endif)/m, "")

  source += group_list(outer.scan(/(uniform|attribute|varying|const) (#{datatypes}) (\w+)(.*?);/))

  data.split(/(#if.*?#endif)/m).each { |chunk|
    # Do the same thing inside #ifdefs
    if chunk[/uniform|attribute|varying|const/]
      if chunk.start_with?("#if")
        tokens = chunk.split(/(.*?\n)(.*?)(#endif)/m)

        source += tokens[1]
        source += group_list(tokens[2].scan(/(uniform|attribute|varying|const) (#{datatypes}) (\w+)(.*?);/))
        source += tokens[2].gsub(/(uniform|attribute|varying|const) (#{datatypes}) (\w+)(.*?);/, "")
        source += tokens[3]
        source += "\n"
      else
        source += chunk.gsub(/(uniform|attribute|varying|const) (#{datatypes}) (\w+)(.*?);/, "")
      end
    else
      source += chunk
    end
  }

  return source
end

# Minify shaders given in an array
def minify_sources(shaders, rewriteall)
  names = [*("A".."Z"), *("AA".."ZZ"), *("aA".."zZ"), *("Aa".."Zz"), *("Aa".."ZZ"), *("A0".."Z9")]
  datatypes = ["float","double","u?int","void","bool","d?mat[2-4](?:x[2-4])?","[ibdu]?vec[2-4]","[iu]?(?:sampler|image)(?:[1-3]D|Cube|Buffer)(?:MSArray|MS|RectShadow|Rect|ArrayShadow|Shadow|Array)?"]
  language_words = ["attribute","const","uniform","varying","buffer","shared","coherent","volatile","restrict","readonly","writeonly","atomic_uint","layout","centroid","flat","smooth","noperspective","patch","sample","break","continue","do","for","while","switch","case","default","if","else","subroutine","in","out","inout","float","double","int","void","bool","true","false","invariant","discard","return","mat2","mat3","mat4","dmat2","dmat3","dmat4","mat2x2","mat2x3","mat2x4","dmat2x2","dmat2x3","dmat2x4","mat3x2","mat3x3","mat3x4","dmat3x2","dmat3x3","dmat3x4","mat4x2","mat4x3","mat4x4","dmat4x2","dmat4x3","dmat4x4","vec2","vec3","vec4","ivec2","ivec3","ivec4","bvec2","bvec3","bvec4","dvec2","dvec3","dvec4","uint","uvec2","uvec3","uvec4","lowp","mediump","highp","precision","sampler1D","sampler2D","sampler3D","samplerCube","sampler1DShadow","sampler2DShadow","samplerCubeShadow","sampler1DArray","sampler2DArray","sampler1DArrayShadow","sampler2DArrayShadow","isampler1D","isampler2D","isampler3D","isamplerCube","isampler1DArray","isampler2DArray","usampler1D","usampler2D","usampler3D","u","samplerCube","usampler1DArray","usampler2DArray","sampler2DRect","sampler2DRectShadow","isampler2DRect","usampler2DRect","samplerBuffer","isamplerBuffer","usamplerBuffer","sampler2DMS","isampler2DMS","usampler2DMS","sampler2DMSArray","isampler2DMSArray","usampler2DMSArray","samplerCubeArray","samplerCubeArrayShadow","isamplerCubeArray","usamplerCubeArray","image1D","iimage1D","uimage1D","image2D","iimage2D","uimage2D","image3D","iimage3D","uimage3D","image2DRect","iimage2DRect","uimage2DRect","imageCube","iimageCube","uimageCube","imageBuffer","iimageBuffer","uimageBuffer","image1DArray","iimage1DArray","uimage1DArray","image2DArray","iimage2DArray","uimage2DArray","imageCubeArray","iimageCubeArray","uimageCubeArray","image2DMS","iimage2DMS","uimage2DMS","image2DMSArray","iimage2DMSArray","uimage2DMSArray","struct","gl_VertexID","gl_InstanceID","gl_PerVertex","gl_Position","gl_PointSize","gl_ClipDistance","gl_PatchVerticesIn","gl_PrimitiveID","gl_InvocationID","gl_TessLevelOuter","gl_TessLevelInner","gl_TessCoord","gl_PrimitiveIDIn","gl_Layer","gl_ViewportIndex","gl_FragCoord","gl_FrontFacing","gl_PointCoord","gl_SampleID","gl_SamplePosition","gl_SampleMaskIn","gl_FragDepth","gl_SampleMask","gl_NumWorkGroups","gl_WorkGroupSize","gl_LocalGroupSize","gl_WorkGroupID","gl_LocalInvocationID","gl_GlobalInvocationID","gl_LocalInvocationIndex","gl_MaxComputeWorkGroupCount","gl_MaxComputeWorkGroupSize","gl_MaxComputeUniformComponents","gl_MaxComputeTextureImageUnits","gl_MaxComputeImageUniforms","gl_MaxComputeAtomicCounters","gl_MaxComputeAtomicCounterBuffers","gl_MaxVertexAttribs","gl_MaxVertexUniformComponents","gl_MaxVaryingComponents","gl_MaxVertexOutputComponents","gl_MaxGeometryInputComponents","gl_MaxGeometryOutputComponents","gl_MaxFragmentInputComponents","gl_MaxVertexTextureImageUnits","gl_MaxCombinedTextureImageUnits","gl_MaxTextureImageUnits","gl_MaxImageUnits","gl_MaxCombinedImageUnitsAndFragmentOutputs","gl_MaxImageSamples","gl_MaxVertexImageUniforms","gl_MaxTessControlImageUniforms","gl_MaxTessEvaluationImageUniforms","gl_MaxGeometryImageUniforms","gl_MaxFragmentImageUniforms","gl_MaxCombinedImageUniforms","gl_MaxFragmentUniformComponents","gl_MaxDrawBuffers","gl_MaxClipDistances","gl_MaxGeometryTextureImageUnits","gl_MaxGeometryOutputVertices","gl_MaxGeometryTotalOutputComponents","gl_MaxGeometryUniformComponents","gl_MaxGeometryVaryingComponents","gl_MaxTessControlInputComponents","gl_MaxTessControlOutputComponents","gl_MaxTessControlTextureImageUnits","gl_MaxTessControlUniformComponents","gl_MaxTessControlTotalOutputComponents","gl_MaxTessEvaluationInputComponents","gl_MaxTessEvaluationOutputComponents","gl_MaxTessEvaluationTextureImageUnits","gl_MaxTessEvaluationUniformComponents","gl_MaxTessPatchComponents","gl_MaxPatchVertices","gl_MaxTessGenLevel","gl_MaxViewports","gl_MaxVertexUniformVectors","gl_MaxFragmentUniformVectors","gl_MaxVaryingVectors","gl_MaxVertexAtomicCounters","gl_MaxTessControlAtomicCounters","gl_MaxTessEvaluationAtomicCounters","gl_MaxGeometryAtomicCounters","gl_MaxFragmentAtomicCounters","gl_MaxCombinedAtomicCounters","gl_MaxAtomicCounterBindings","gl_MaxVertexAtomicCounterBuffers","gl_MaxTessControlAtomicCounterBuffers","gl_MaxTessEvaluationAtomicCounterBuffers","gl_MaxGeometryAtomicCounterBuffers","gl_MaxFragmentAtomicCounterBuffers","gl_MaxCombinedAtomicCounterBuffers","gl_MaxAtomicCounterBufferSize","gl_MinProgramTexelOffset","gl_MaxProgramTexelOffset","gl_MaxTransformFeedbackBuffers","gl_MaxTransformFeedbackInterleavedComponents","radians","degrees","sin","cos","tan","asin","acos","atan","sinh","cosh","tanh","asinh","acosh","atanh","pow","exp","log","exp2","log2","sqrt","inversesqrt","abs","sign","floor","trunc","round","roundEven","ceil","fract","mod","modf","min","max","clamp","mix","step","smoothstep","isnan","isinf","floatBitsToInt","floatBitsToUint","intBitsToFloat","uintBitsToFloat","fma","frexp","ldexp","packUnorm2x16","packSnorm2x16","packUnorm4x8","packSnorm4x8","unpackUnorm2x16","unpackSnorm2x16","unpackUnorm4x8","unpackSnorm4x8","packDouble2x32","unpackDouble2x32","packHalf2x16","unpackHalf2x16","length","distance","dot","cross","normalize","faceforward","reflect","refract","matrixCompMult","outerProduct","transpose","determinant","inverse","lessThan","lessThanEqual","greaterThan","greaterThanEqual","equal","notEqual","any","all","not","uaddCarry","usubBorrow","umulExtended","imulExtended","bitfieldExtract","bitfieldReverse","bitfieldInsert","bitCount","findLSB","findMSB","atomicCounterIncrement","atomicCounterDecrement","atomicCounter","atomicOP","imageSize","imageLoad","imageStore","imageAtomicAdd","imageAtomicMin","imageAtomicMax","imageAtomicAnd","mageAtomicOr","imageAtomicXor","imageAtomicExchange","imageAtomicCompSwap","dFdx","dFdy","fwidth","interpolateAtCentroid","interpolateAtSample","interpolateAtOffset","noise1","noise2","noise3","noise4","EmitStreamVertex","EndStreamPrimitive","EmitVertex","EndPrimitive","barrier","memoryBarrier","groupMemoryBarrier","memoryBarrierAtomicCounter","memoryBarrierShared","memoryBarrierBuffer","memoryBarrierImage","textureSize","textureQueryLod","textureQueryLevels","texture","textureProj","textureLod","tureOffset","texelFetch","texelFetchOffset","textureProjOffset","textureLodOffset","textureProjLod","textureProjLodOffset","textureGrad","textureGradOffset","textureProjGrad","textureProjGradOffset","textureGather","textureGatherOffset","textureGatherOffsets","texture2D","texture2DProj","texture2DLod","texture2DProjLod","textureCube","textureCubeLod"] + gen_sizzle_permutations()
  structs = []
  defines = []
  functions = []
  uniforms = []
  attributes = []
  varyings = []
  constants = [];
  members = [];

  shaders.map! { |shader|
    remove_comments(shader)
    remove_whitespace(shader)
  }

  # Get struct names
  shaders.each { |shader|
    structs += get_struct_names(shader)
  }

  # Create a regex of all the known data types
  datatypes_string = datatypes.concat(structs).join("|")

  # Remove dead functions that are not in the call graph of the main() function in any of the inputs
  remove_dead_functions(shaders, datatypes_string)

  # Get all function/uniform/attribute/varying names, and define names and their values
  shaders.each { |shader|
    functions += get_function_names(shader, datatypes_string)
    uniforms += get_variable_names(shader, "uniform", datatypes_string)
    attributes += get_variable_names(shader, "attribute", datatypes_string)
    varyings += get_variable_names(shader, "varying", datatypes_string)
    constants += get_variable_names(shader, "const", datatypes_string)
    defines += get_defines(shader)
    members += get_member_names(shader, datatypes_string);
  }

  function_map = gen_map(functions, names, true)
  struct_map = gen_map(structs, names, true)
  uniform_map = gen_map(uniforms, names, rewriteall)
  attribute_map = gen_map(attributes, names, rewriteall)
  varyings_map = gen_map(varyings, names, true)
  constants_map = gen_map(constants, names, true)
  member_map = gen_map_map(members, [*("a".."z"), *("A".."Z"), *("aa".."zz")], rewriteall)

  language_words = language_words.uniq().map { |v|
    [v, names.shift()]
  }

  # Preprocess #defines to prepare them for inlining
  preprocess_defines(defines)

  shaders.map! { |shader|
    # Inline #defines
    inline_defines(defines, shader)

    shader = group_globals(shader, datatypes_string)

    # Rewrite function names
    function_map.each { |function|
      shader.gsub!(/\b#{function[0]}\b/, function[1])
    }

    # Rename function arguments and local variables
    shader = rename_locals(shader, datatypes_string)

    # Rewrite user defined type names
    struct_map.each { |struct|
      shader.gsub!(/\b#{struct[0]}\b/, struct[1])
    }

    # Rewrite uniform names
    rewrite_map(uniform_map, shader)

    # Rewrite attribute names
    rewrite_map(attribute_map, shader)

    # Rewrite varying names
    rewrite_map(varyings_map, shader)

    # Rewrite varying names
    rewrite_map(constants_map, shader)

    # Rewrite struct member names
    shader = rename_members(shader, member_map, datatypes_string)

    # Rewrite numbers
    rewrite_numbers(shader)

    shader = add_defines(shader, language_words)

    shader = remove_whitespace(shader).gsub("\n", "\\n")

    # If the first line of a shader is a pre-processor directive, it will cause an error when concatenating it, so add a new line
    if shader[0] == "#"
      shader = "\\n" + shader
    end

    shader
  }

  global_map = {}

  uniform_map.concat(attribute_map).each { |v|
    global_map[v[0]] = v[1]
  }

  return [shaders, global_map, member_map]
end

def minify_files(paths, rewriteall)
  minify_sources(paths.map { |path| IO.read(path) }, rewriteall)
end

Dir["**/*.{vert,frag}"].each { |file|
  next if file.include? ".min."
  new_path = "#{file[0..-5]}min#{file[-5..]}"
  min_src = minify_files([file], true)[0][0].gsub("\\n", "\n")
  File.write(new_path, min_src)
}
