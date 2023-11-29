all:
	 docker build -t mypsql --build-arg="PG_MAJOR=16" .
	 
run:
	docker run -p 5432:5432 mypsql