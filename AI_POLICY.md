# AI/LLM Policy

LLM may be used for the following:

- Chat discussion/review of project design
- Trivial single-line code completion
- Debugging and code review
- Analyzing code, test results and benchmark results,
  provided the content is not directly committed to the repo
- Generating scripts for processing/refactoring code,
  provided the script is not committed to the repo and
  both the script and the changes are thoroughly reviewed

LLM should not be used for the following:

- Writing design documents
- Generating more than a few lines of trivial code
- Direct generation of non-trivial code
- Massive refactoring without line-by-line human review
- Direct, unverified derivation of formulas
- Deciding unit test cases

Contributors are expected not to use LLM for tasks beyond those allowed above.
