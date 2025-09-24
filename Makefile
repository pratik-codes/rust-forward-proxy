# Makefile for Rust Forward Proxy
# ================================

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
	@echo "$(CYAN)Rust Forward Proxy$(NC)"
	@echo "==================="
	@echo ""
	@echo "$(YELLOW)Main Commands:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E "^(dev|prod)" | grep -v docker-detached | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)  %-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Setup & Testing:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E "^(setup|test)" | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)  %-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Docker Commands:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E "^(docker|build|pull)" | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)  %-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Help & Utilities:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E "^(help|cache|status|health|clean|backup|restore)" | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)  %-20s$(NC) %s\n", $$1, $$2}'
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
# MAIN DEVELOPMENT COMMANDS
# =================================================

.PHONY: dev
dev: setup ## Start local development with HTTPS interception (use CERT=securly for Securly CA)
	@echo "$(BLUE)Starting local development with HTTPS interception...$(NC)"
	@if [ ! -f .env ]; then \
		echo "$(YELLOW)Creating .env file from template...$(NC)"; \
		cp env.example .env; \
		echo "$(GREEN)✅ Created .env file$(NC)"; \
	fi
	@mkdir -p logs certs
	@# Use CERT environment variable to select certificate type
	@CERT_TYPE="$${CERT:-rootca}"; \
	echo "$(CYAN)Selected certificate type: $$CERT_TYPE$(NC)"; \
	if [ "$$CERT_TYPE" = "securly" ]; then \
		CA_CERT_PATH="ca-certs/securly_ca.crt"; \
		CA_KEY_PATH="ca-certs/securly_ca.key"; \
		echo "$(GREEN)🔒 Starting proxy with Securly CA certificates...$(NC)"; \
		echo "$(CYAN)📜 CA Certificate: $$CA_CERT_PATH$(NC)"; \
		if [ ! -f "$$CA_KEY_PATH" ]; then \
			echo "$(YELLOW)⚠️  Warning: securly_ca.key not found - will fallback to self-signed certificates$(NC)"; \
		fi; \
		echo "$(YELLOW)⚠️  Make sure to install securly_ca.crt in your browser$(NC)"; \
	else \
		CA_CERT_PATH="ca-certs/rootCA.crt"; \
		CA_KEY_PATH="ca-certs/rootCA.key"; \
		echo "$(GREEN)🔒 Starting proxy with rootCA certificates...$(NC)"; \
		echo "$(CYAN)📜 CA Certificate: $$CA_CERT_PATH$(NC)"; \
		echo "$(YELLOW)⚠️  Make sure to install rootCA.crt in your browser$(NC)"; \
	fi; \
	echo "$(CYAN)Environment: Local development with HTTPS interception$(NC)"; \
	echo "$(CYAN)Proxy: http://127.0.0.1:8080$(NC)"; \
	echo "$(CYAN)🔍 HTTP requests: INFO level logging$(NC)"; \
	echo "$(CYAN)🔍 HTTPS requests: Intercepted and fully logged at INFO level$(NC)"; \
	echo "$(CYAN)🔐 CONNECT requests: DEBUG level only (hidden at INFO)$(NC)"; \
	echo "$(CYAN)📁 Logs: Console + File (logs/proxy.log.YYYY-MM-DD)$(NC)"; \
	echo "$(CYAN)Test HTTP: curl -x http://127.0.0.1:8080 http://httpbin.org/get$(NC)"; \
	echo "$(CYAN)Test HTTPS: curl -x http://127.0.0.1:8080 https://httpbin.org/get --proxy-insecure$(NC)"; \
	echo "$(CYAN)Debug mode: RUST_LOG=debug make dev$(NC)"; \
	echo "$(CYAN)Usage: CERT=securly make dev$(NC)"; \
	PROXY_LISTEN_ADDR=127.0.0.1:8080 \
	 HTTPS_INTERCEPTION_ENABLED=true \
	 TLS_ENABLED=false \
	 TLS_CA_CERT_PATH="$$CA_CERT_PATH" \
	 TLS_CA_KEY_PATH="$$CA_KEY_PATH" \
	 UPSTREAM_URL=http://httpbin.org \
	 RUST_LOG=info \
	 cargo run --bin rust-forward-proxy --no-default-features

.PHONY: dev-securly
dev-securly: setup ## Start local development with Securly CA certificates
	@echo "$(GREEN)🔒 Starting with Securly CA certificates...$(NC)"
	@CERT=securly $(MAKE) dev

.PHONY: dev-docker
dev-docker: setup ## Start development environment with Docker
	@echo "$(BLUE)Starting development environment with Docker...$(NC)"
	$(DOCKER_COMPOSE) up --build

.PHONY: dev-docker-detached
dev-docker-detached: setup ## Start Docker development environment in background
	@echo "$(BLUE)Starting Docker development environment in background...$(NC)"
	$(DOCKER_COMPOSE) up -d --build
	@make status

.PHONY: setup-ca
setup-ca: ## Generate root CA certificate for browser installation
	@echo "$(BLUE)Setting up Root CA for HTTPS Interception...$(NC)"
	@./scripts/setup_ca.sh
	@echo ""
	@echo "$(GREEN)📖 Next steps: See BROWSER_SETUP.md for complete browser configuration guide$(NC)"

.PHONY: help-browser
help-browser: ## Show browser setup instructions
	@echo "$(GREEN)🌐 Browser HTTPS Interception Setup$(NC)"
	@echo "======================================"
	@echo ""
	@echo "$(CYAN)Quick Setup:$(NC)"
	@echo "1. $(YELLOW)make setup-ca$(NC)                    # Generate root certificate"
	@echo "2. Install certificate in browser    # See BROWSER_SETUP.md" 
	@echo "3. Configure proxy: 127.0.0.1:8080   # In browser settings"
	@echo "4. $(YELLOW)make dev$(NC)                         # Start intercepting proxy"
	@echo ""
	@echo "$(GREEN)📖 Complete guide: BROWSER_SETUP.md$(NC)"
	@echo ""

.PHONY: help-cache
help-cache: ## Show certificate caching information
	@echo "$(GREEN)🔒 Certificate Caching System$(NC)"
	@echo "==============================="
	@echo ""
	@echo "$(CYAN)Performance Benefits:$(NC)"
	@echo "• First request:  Generate certificate (~10ms)"
	@echo "• Later requests: Use cached cert (<1ms)"
	@echo "• 25-30x faster HTTPS interception!"
	@echo ""
	@echo "$(CYAN)Cache Backends:$(NC)"
	@echo "• $(YELLOW)Local$(NC):      In-memory cache (1000 certs max)"
	@echo "• $(YELLOW)Docker$(NC):     Redis cache (unlimited, shared)"
	@echo ""
	@echo "$(CYAN)Commands:$(NC)"
	@echo "• $(YELLOW)make dev$(NC)                  # Test with memory cache"
	@echo "• $(YELLOW)make dev-docker$(NC)           # Test with Redis cache"
	@echo "• $(YELLOW)make cache-clear-redis$(NC)    # Clear Redis certificate cache"
	@echo ""
	@echo "$(GREEN)📖 Full documentation: CERTIFICATE_CACHING.md$(NC)"
	@echo ""

.PHONY: help-logging
help-logging: ## Show logging configuration help
	@echo "$(GREEN)📋 Logging Configuration$(NC)"
	@echo "=========================="
	@echo ""
	@echo "$(CYAN)Log Levels:$(NC)"
	@echo "• $(YELLOW)INFO$(NC):   HTTP/HTTPS requests visible, CONNECT hidden"
	@echo "• $(YELLOW)DEBUG$(NC):  All requests visible including CONNECT"
	@echo ""
	@echo "$(CYAN)Examples:$(NC)"
	@echo "• $(YELLOW)make dev$(NC)                  # INFO level (production-like)"
	@echo "• $(YELLOW)RUST_LOG=debug make dev$(NC)   # DEBUG level (verbose)"
	@echo "• $(YELLOW)RUST_LOG=trace make dev$(NC)   # TRACE level (very verbose)"
	@echo ""
	@echo "$(GREEN)📖 Full documentation: LOGGING_CHANGES.md$(NC)"
	@echo ""

.PHONY: cache-clear-redis
cache-clear-redis: ## Clear Redis certificate cache
	@echo "$(BLUE)Clearing Redis certificate cache...$(NC)"
	@if command -v redis-cli >/dev/null 2>&1; then \
		if redis-cli ping >/dev/null 2>&1; then \
			KEYS=$$(redis-cli --scan --pattern "proxy:cert:*" | wc -l); \
			if [ "$$KEYS" -gt 0 ]; then \
				redis-cli --scan --pattern "proxy:cert:*" | xargs redis-cli del; \
				echo "$(GREEN)✅ Cleared $$KEYS cached certificates$(NC)"; \
			else \
				echo "$(YELLOW)ℹ️  No certificates found in Redis cache$(NC)"; \
			fi; \
		else \
			echo "$(RED)❌ Redis server not running$(NC)"; \
		fi; \
	else \
		echo "$(RED)❌ redis-cli not found. Install Redis tools first.$(NC)"; \
	fi

# =================================================
# MAIN PRODUCTION COMMANDS
# =================================================

.PHONY: prod
prod: setup-prod ## Start local production server
	@echo "$(BLUE)Starting local production server...$(NC)"
	@if [ ! -f .env ]; then \
		echo "$(YELLOW)Creating .env file from template...$(NC)"; \
		cp env.example .env; \
		echo "$(GREEN)✅ Created .env file$(NC)"; \
	fi
	@mkdir -p logs certs
	@echo "$(GREEN)🚀 Starting production proxy server...$(NC)"
	@echo "$(CYAN)Environment: Local production$(NC)"
	@echo "$(CYAN)Proxy: http://127.0.0.1:8080$(NC)"
	@echo "$(CYAN)Log Level: INFO (CONNECT requests hidden, HTTP/HTTPS visible)$(NC)"
	@echo "$(CYAN)HTTPS Interception: Enabled$(NC)"
	@echo "$(CYAN)Test HTTP: curl -x http://127.0.0.1:8080 http://httpbin.org/get$(NC)"
	@echo "$(CYAN)Test HTTPS: curl -x http://127.0.0.1:8080 https://httpbin.org/get --proxy-insecure$(NC)"
	@PROXY_LISTEN_ADDR=127.0.0.1:8080 \
	 HTTPS_INTERCEPTION_ENABLED=true \
	 TLS_ENABLED=false \
	 UPSTREAM_URL=http://httpbin.org \
	 RUST_LOG=info \
	 cargo run --bin rust-forward-proxy --release --no-default-features

.PHONY: prod-docker
prod-docker: setup-prod ## Start production environment with Docker
	@echo "$(BLUE)Starting production environment with Docker...$(NC)"
	$(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml up -d --build
	@make status

.PHONY: prod-docker-deploy
prod-docker-deploy: setup-prod ## Deploy to production with Docker (pull latest images)
	@echo "$(BLUE)Deploying to production with Docker...$(NC)"
	$(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml pull
	$(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml up -d --build
	@make status

# =================================================
# DOCKER UTILITY COMMANDS
# =================================================

.PHONY: docker-stop
docker-stop: ## Stop all Docker services
	@echo "$(YELLOW)Stopping Docker services...$(NC)"
	$(DOCKER_COMPOSE) stop

.PHONY: docker-down
docker-down: ## Stop and remove Docker containers
	@echo "$(YELLOW)Stopping and removing Docker containers...$(NC)"
	$(DOCKER_COMPOSE) down

.PHONY: docker-down-volumes
docker-down-volumes: ## Stop and remove Docker containers with volumes
	@echo "$(RED)Stopping and removing Docker containers with volumes...$(NC)"
	@echo "$(YELLOW)⚠️  This will delete all Redis data!$(NC)"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo ""; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		$(DOCKER_COMPOSE) down -v; \
	else \
		echo "$(GREEN)Cancelled.$(NC)"; \
	fi

.PHONY: docker-restart
docker-restart: ## Restart Docker services
	@echo "$(BLUE)Restarting Docker services...$(NC)"
	$(DOCKER_COMPOSE) restart
	@make status

.PHONY: docker-logs
docker-logs: ## Show logs from all Docker services
	$(DOCKER_COMPOSE) logs -f

.PHONY: docker-logs-proxy
docker-logs-proxy: ## Show logs from proxy service only
	$(DOCKER_COMPOSE) logs -f rust-proxy

.PHONY: docker-logs-redis
docker-logs-redis: ## Show logs from Redis service only
	$(DOCKER_COMPOSE) logs -f redis

.PHONY: status
status: ## Show Docker service status
	@echo "$(BLUE)Docker Service Status:$(NC)"
	$(DOCKER_COMPOSE) ps

# =================================================
# TESTING COMMANDS
# =================================================

.PHONY: test
test: test-local ## Run default proxy tests (alias for test-local)

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
test-all: test-local test-intercept test-docker test-redis ## Run all tests

.PHONY: test-local
test-local: ## Test local proxy
	@echo "$(BLUE)Testing local proxy...$(NC)"
	@echo "$(YELLOW)Make sure 'make dev' is running in another terminal first$(NC)"
	@if curl -s -f --max-time 10 -x http://127.0.0.1:8080 http://httpbin.org/get > /dev/null; then \
		echo "$(GREEN)✅ Local proxy test passed$(NC)"; \
	else \
		echo "$(RED)❌ Local proxy test failed$(NC)"; \
		echo "$(YELLOW)💡 Make sure to run 'make dev' in another terminal first$(NC)"; \
		exit 1; \
	fi

.PHONY: test-intercept
test-intercept: ## Test HTTPS interception proxy
	@echo "$(BLUE)Testing HTTPS interception proxy...$(NC)"
	@echo "$(YELLOW)Make sure 'make dev' is running in another terminal first$(NC)"
	@echo "$(CYAN)Testing HTTP request...$(NC)"
	@if curl -s -f --max-time 10 -x http://127.0.0.1:8080 http://httpbin.org/get > /dev/null; then \
		echo "$(GREEN)✅ HTTP test passed$(NC)"; \
	else \
		echo "$(RED)❌ HTTP test failed$(NC)"; \
		exit 1; \
	fi
	@echo "$(CYAN)Testing HTTPS request (with interception)...$(NC)"
	@if curl -s -f --max-time 10 --proxy-insecure -x http://127.0.0.1:8080 https://httpbin.org/get > /dev/null; then \
		echo "$(GREEN)✅ HTTPS interception test passed$(NC)"; \
		echo "$(GREEN)🔍 Check proxy logs - you should see detailed HTTP/HTTPS content!$(NC)"; \
	else \
		echo "$(RED)❌ HTTPS interception test failed$(NC)"; \
		exit 1; \
	fi

.PHONY: test-docker
test-docker: ## Test Docker proxy
	@echo "$(BLUE)Testing Docker proxy...$(NC)"
	@PROXY_PORT=$$(grep PROXY_PORT .env 2>/dev/null | cut -d'=' -f2 | tr -d '"' || echo "8080"); \
	echo "$(YELLOW)Testing Docker proxy on port $$PROXY_PORT...$(NC)"; \
	if curl -s -f --max-time 10 -x http://localhost:$$PROXY_PORT http://httpbin.org/get > /dev/null; then \
		echo "$(GREEN)✅ Docker proxy test passed$(NC)"; \
	else \
		echo "$(RED)❌ Docker proxy test failed$(NC)"; \
		exit 1; \
	fi

# =================================================
# MAINTENANCE COMMANDS
# =================================================

.PHONY: clean
clean: docker-down ## Clean up Docker containers and images
	@echo "$(YELLOW)Cleaning up Docker resources...$(NC)"
	docker system prune -f
	@echo "$(GREEN)✅ Cleanup complete$(NC)"

.PHONY: clean-all
clean-all: docker-down-volumes ## Clean up everything including volumes and images
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
# DOCKER BUILD & DEPLOYMENT
# =================================================

.PHONY: build
build: ## Build Docker containers without starting
	$(DOCKER_COMPOSE) build

.PHONY: pull
pull: ## Pull latest Docker images
	$(DOCKER_COMPOSE) pull

.PHONY: docker-shell-proxy
docker-shell-proxy: ## Open shell in proxy container
	$(DOCKER_COMPOSE) exec rust-proxy /bin/sh

.PHONY: docker-shell-redis
docker-shell-redis: ## Open Redis CLI
	$(DOCKER_COMPOSE) exec redis redis-cli

.PHONY: docker-config
docker-config: ## Show resolved Docker Compose configuration
	$(DOCKER_COMPOSE) config

.PHONY: docker-config-prod
docker-config-prod: ## Show resolved production Docker Compose configuration
	$(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml config

# =================================================
# MONITORING
# =================================================

.PHONY: health
health: ## Check health of Docker services
	@echo "$(BLUE)Checking Docker service health...$(NC)"
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

.PHONY: docker-stats
docker-stats: ## Show Docker container stats
	docker stats --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.PIDs}}"

# Note: Phony targets to prevent conflicts with files of the same name
.PHONY: all
all: help
