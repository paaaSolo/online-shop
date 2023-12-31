up:
	docker-compose up -d --build

down:
	docker-compose down

logs:
	docker-compose logs -f

build:
	docker build . -t app
