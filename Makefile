all:
	 docker build -t mypsql --build-arg="PG_MAJOR=16" .