all:
	 docker build -t postgresql-ulid --build-arg="PG_MAJOR=16" .
	 
run:
	docker run -p 5432:5432 postgresql-ulid