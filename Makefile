all:
	 docker build --build-arg="PG_MAJOR=16" .