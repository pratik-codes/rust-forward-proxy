# Makefile for Rust Forward Proxy Docker Management
# =================================================

# Default environment
ENV ?= dev

# Docker Compose command detection
DOCKER_COMPOSE := $(shell command -v docker-compose 2> /dev/null)
ifndef DOCKER_COMPOSE
	DOCKER_COMPOSE := docker compose
endif

# Colors for output
RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[0;33m
BLUE = \033[0;34m
PURPLE = \033[0;35m
CYAN = \033[0;36m
NC = \033[0m # No Color

# Default target
.DEFAULT_GOAL := help

# =================================================
# HELP
# =================================================

.PHONY: help
help: ## Show this help message
	@echo ""
	@echo "$(CYAN)Rust Forward Proxy - Docker Management$(NC)"
	@echo "======================================"
	@echo ""
	@echo "$(YELLOW)Development Commands:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E "(dev|setup|test)" | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)  %-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Production Commands:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E "(prod|deploy)" | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)  %-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Utility Commands:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -vE "(dev|prod|setup|test|deploy)" | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)  %-20s$(NC) %s\n", $$1, $$2}'
	@echo ""

# =================================================
# SETUP COMMANDS
# =================================================

.PHONY: setup
setup: ## Initial setup - create .env file and prepare environment
	@echo "$(BLUE)Setting up development environment...$(NC)"
	@if [ ! -f .env ]; then \
		echo "$(YELLOW)Creating .env file from template...$(NC)"; \
		cp env.example .env; \
		echo "$(GREEN)✅ Created .env file$(NC)"; \
		echo "$(YELLOW)⚠️  Please edit .env and set REDIS_PASSWORD to a secure value$(NC)"; \
	else \
		echo "$(GREEN)✅ .env file already exists$(NC)"; \
	fi
	@mkdir -p logs
	@echo "$(GREEN)✅ Setup complete!$(NC)"

.PHONY: setup-prod
setup-prod: ## Setup production environment with secure password generation
	@echo "$(BLUE)Setting up production environment...$(NC)"
	@if [ ! -f .env ]; then \
		cp env.example .env; \
		REDIS_PASSWORD=$$(openssl rand -base64 32 2>/dev/null || python3 -c "import secrets, string; print(''.join(secrets.choice(string.ascii_letters + string.digits + '!@#$$%^&*') for _ in range(24)))" 2>/dev/null || echo "SecurePassword$$(date +%s)!"); \
		if command -v sed >/dev/null 2>&1; then \
			sed -i.bak "s/your_secure_redis_password_here/$$REDIS_PASSWORD/" .env; \
			rm -f .env.bak; \
			echo "$(GREEN)✅ Generated secure Redis password$(NC)"; \
		else \
			echo "$(YELLOW)⚠️  Please manually edit .env and set REDIS_PASSWORD$(NC)"; \
		fi; \
	else \
		echo "$(GREEN)✅ .env file already exists$(NC)"; \
	fi
	@mkdir -p logs
	@echo "$(GREEN)✅ Production setup complete!$(NC)"

# =================================================
# DEVELOPMENT COMMANDS
# =================================================

.PHONY: dev
dev: setup ## Start development environment
	@echo "$(BLUE)Starting development environment...$(NC)"
	$(DOCKER_COMPOSE) up --build

.PHONY: dev-detached
dev-detached: setup ## Start development environment in background
	@echo "$(BLUE)Starting development environment in background...$(NC)"
	$(DOCKER_COMPOSE) up -d --build
	@make status

.PHONY: dev-rebuild
dev-rebuild: ## Rebuild and start development environment
	@echo "$(BLUE)Rebuilding development environment...$(NC)"
	$(DOCKER_COMPOSE) up --build --force-recreate

# =================================================
# PRODUCTION COMMANDS
# =================================================

.PHONY: prod
prod: setup-prod ## Start production environment
	@echo "$(BLUE)Starting production environment...$(NC)"
	$(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml up -d --build
	@make status

.PHONY: prod-deploy
prod-deploy: setup-prod ## Deploy to production (pull latest images)
	@echo "$(BLUE)Deploying to production...$(NC)"
	$(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml pull
	$(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml up -d --build
	@make status

.PHONY: prod-restart
prod-restart: ## Restart production services
	@echo "$(BLUE)Restarting production services...$(NC)"
	$(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml restart
	@make status

# =================================================
# UTILITY COMMANDS
# =================================================

.PHONY: stop
stop: ## Stop all services
	@echo "$(YELLOW)Stopping services...$(NC)"
	$(DOCKER_COMPOSE) stop

.PHONY: down
down: ## Stop and remove containers
	@echo "$(YELLOW)Stopping and removing containers...$(NC)"
	$(DOCKER_COMPOSE) down

.PHONY: down-volumes
down-volumes: ## Stop and remove containers with volumes
	@echo "$(RED)Stopping and removing containers with volumes...$(NC)"
	@echo "$(YELLOW)⚠️  This will delete all Redis data!$(NC)"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo ""; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		$(DOCKER_COMPOSE) down -v; \
	else \
		echo "$(GREEN)Cancelled.$(NC)"; \
	fi

.PHONY: restart
restart: ## Restart all services
	@echo "$(BLUE)Restarting services...$(NC)"
	$(DOCKER_COMPOSE) restart
	@make status

.PHONY: status
status: ## Show service status
	@echo "$(BLUE)Service Status:$(NC)"
	$(DOCKER_COMPOSE) ps

.PHONY: logs
logs: ## Show logs from all services
	$(DOCKER_COMPOSE) logs -f

.PHONY: logs-proxy
logs-proxy: ## Show logs from proxy service only
	$(DOCKER_COMPOSE) logs -f rust-proxy

.PHONY: logs-redis
logs-redis: ## Show logs from Redis service only
	$(DOCKER_COMPOSE) logs -f redis

# =================================================
# TESTING COMMANDS
# =================================================

.PHONY: test
test: ## Run basic proxy tests
	@echo "$(BLUE)Testing proxy functionality...$(NC)"
	@PROXY_PORT=$$(grep PROXY_PORT .env 2>/dev/null | cut -d'=' -f2 | tr -d '"' || echo "8080"); \
	echo "$(YELLOW)Testing proxy on port $$PROXY_PORT...$(NC)"; \
	if curl -s -f --max-time 10 -x http://localhost:$$PROXY_PORT http://httpbin.org/get > /dev/null; then \
		echo "$(GREEN)✅ Proxy test passed$(NC)"; \
	else \
		echo "$(RED)❌ Proxy test failed$(NC)"; \
		exit 1; \
	fi

.PHONY: test-redis
test-redis: ## Test Redis connection
	@echo "$(BLUE)Testing Redis connection...$(NC)"
	@if $(DOCKER_COMPOSE) exec -T redis redis-cli ping > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Redis connection test passed$(NC)"; \
	else \
		echo "$(RED)❌ Redis connection test failed$(NC)"; \
		exit 1; \
	fi

.PHONY: test-all
test-all: test test-redis ## Run all tests

# =================================================
# MAINTENANCE COMMANDS
# =================================================

.PHONY: clean
clean: down ## Clean up containers and images
	@echo "$(YELLOW)Cleaning up Docker resources...$(NC)"
	docker system prune -f
	@echo "$(GREEN)✅ Cleanup complete$(NC)"

.PHONY: clean-all
clean-all: down-volumes ## Clean up everything including volumes and images
	@echo "$(RED)Cleaning up all Docker resources...$(NC)"
	@echo "$(YELLOW)⚠️  This will delete all data and images!$(NC)"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo ""; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		docker system prune -a -f --volumes; \
		echo "$(GREEN)✅ Complete cleanup done$(NC)"; \
	else \
		echo "$(GREEN)Cancelled.$(NC)"; \
	fi

.PHONY: backup-redis
backup-redis: ## Backup Redis data
	@echo "$(BLUE)Creating Redis backup...$(NC)"
	@mkdir -p backups
	@TIMESTAMP=$$(date +%Y%m%d_%H%M%S); \
	$(DOCKER_COMPOSE) exec -T redis redis-cli BGSAVE; \
	sleep 2; \
	docker cp rust-proxy-redis:/data/dump.rdb ./backups/redis_backup_$$TIMESTAMP.rdb; \
	echo "$(GREEN)✅ Redis backup created: backups/redis_backup_$$TIMESTAMP.rdb$(NC)"

.PHONY: restore-redis
restore-redis: ## Restore Redis data from backup (interactive)
	@echo "$(BLUE)Available Redis backups:$(NC)"
	@ls -la backups/redis_backup_*.rdb 2>/dev/null || echo "$(YELLOW)No backups found$(NC)"
	@read -p "Enter backup filename (from backups/): " backup_file; \
	if [ -f "backups/$$backup_file" ]; then \
		echo "$(YELLOW)Stopping Redis...$(NC)"; \
		$(DOCKER_COMPOSE) stop redis; \
		echo "$(BLUE)Restoring backup...$(NC)"; \
		docker cp "./backups/$$backup_file" rust-proxy-redis:/data/dump.rdb; \
		echo "$(BLUE)Starting Redis...$(NC)"; \
		$(DOCKER_COMPOSE) start redis; \
		echo "$(GREEN)✅ Redis restored from $$backup_file$(NC)"; \
	else \
		echo "$(RED)❌ Backup file not found$(NC)"; \
	fi

# =================================================
# DEVELOPMENT UTILITIES
# =================================================

.PHONY: shell-proxy
shell-proxy: ## Open shell in proxy container
	$(DOCKER_COMPOSE) exec rust-proxy /bin/sh

.PHONY: shell-redis
shell-redis: ## Open Redis CLI
	$(DOCKER_COMPOSE) exec redis redis-cli

.PHONY: build
build: ## Build containers without starting
	$(DOCKER_COMPOSE) build

.PHONY: pull
pull: ## Pull latest images
	$(DOCKER_COMPOSE) pull

.PHONY: config
config: ## Show resolved Docker Compose configuration
	$(DOCKER_COMPOSE) config

.PHONY: config-prod
config-prod: ## Show resolved production Docker Compose configuration
	$(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml config

# =================================================
# MONITORING
# =================================================

.PHONY: health
health: ## Check health of all services
	@echo "$(BLUE)Checking service health...$(NC)"
	@echo "$(YELLOW)Docker Compose Status:$(NC)"
	@$(DOCKER_COMPOSE) ps
	@echo ""
	@echo "$(YELLOW)Redis Health:$(NC)"
	@if $(DOCKER_COMPOSE) exec -T redis redis-cli ping > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Redis: Healthy$(NC)"; \
	else \
		echo "$(RED)❌ Redis: Unhealthy$(NC)"; \
	fi
	@echo ""
	@echo "$(YELLOW)Proxy Health:$(NC)"
	@PROXY_PORT=$$(grep PROXY_PORT .env 2>/dev/null | cut -d'=' -f2 | tr -d '"' || echo "8080"); \
	if curl -s -f --max-time 5 http://localhost:$$PROXY_PORT/health > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Proxy: Healthy$(NC)"; \
	else \
		echo "$(RED)❌ Proxy: Unhealthy$(NC)"; \
	fi

.PHONY: stats
stats: ## Show Docker container stats
	docker stats --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.PIDs}}"

# =================================================
# QUICK COMMANDS
# =================================================

.PHONY: up
up: dev-detached ## Quick start (alias for dev-detached)

.PHONY: start
start: dev-detached ## Quick start (alias for dev-detached)

# Note: Phony targets to prevent conflicts with files of the same name
.PHONY: all
all: help
