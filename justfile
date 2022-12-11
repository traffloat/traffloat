default: client-run-dev

tokei:
	tokei -C -e "*lock*" -e "*.svg"
