# Makefile for Rust Forward Proxy
# ================================

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
	@echo "$(YELLOW)Application Commands:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E "^(dev|run|build|list-process)" | grep -v docker | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)  %-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Docker Commands:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E "^docker" | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)  %-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Utilities:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E "^(test|clean|status|help)" | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)  %-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(CYAN)Configuration: All settings come from config.yml$(NC)"
	@echo "$(CYAN)• Set use_privileged_ports: true/false for port selection$(NC)"
	@echo "$(CYAN)• Set runtime.mode for single_threaded/multi_threaded/multi_process$(NC)"
	@echo "$(CYAN)• Set runtime.process_count for number of processes$(NC)"
	@echo ""

# =================================================
# APPLICATION COMMANDS
# =================================================

.PHONY: dev
dev: ## Run development mode (auto-detects sudo for privileged ports)
	@echo "$(BLUE)Starting Rust Forward Proxy...$(NC)"
	@echo "$(CYAN)📋 Configuration: config.yml$(NC)"
	@echo "$(CYAN)📁 Logs: Console + File (logs/)$(NC)"
	@mkdir -p logs certs ca-certs
	@# Auto-detect if running as root/sudo and set ports accordingly
	@if [ "$$(id -u)" -eq 0 ]; then \
		echo "$(GREEN)🔒 Running as root - using privileged ports (80/443)$(NC)"; \
		echo "$(YELLOW)⚠️  Setting use_privileged_ports: true$(NC)"; \
		sed -i.bak 's/use_privileged_ports: false/use_privileged_ports: true/' config.yml 2>/dev/null || true; \
		sed -i.bak 's/use_privileged_ports:false/use_privileged_ports: true/' config.yml 2>/dev/null || true; \
	else \
		echo "$(CYAN)🔗 Running as user - using regular ports (8080/8443)$(NC)"; \
		echo "$(YELLOW)⚠️  Setting use_privileged_ports: false$(NC)"; \
		sed -i.bak 's/use_privileged_ports: true/use_privileged_ports: false/' config.yml 2>/dev/null || true; \
		sed -i.bak 's/use_privileged_ports:true/use_privileged_ports: false/' config.yml 2>/dev/null || true; \
	fi; \
	rm -f config.yml.bak; \
	echo "$(CYAN)🧵 Runtime Mode: Set in config.yml$(NC)"; \
	echo ""; \
	cargo run --bin rust-forward-proxy

.PHONY: run
run: ## Run release mode (auto-detects sudo for privileged ports)
	@echo "$(BLUE)Starting Rust Forward Proxy (Release Mode)...$(NC)"
	@echo "$(CYAN)📋 Configuration: config.yml$(NC)"
	@echo "$(CYAN)📁 Logs: Console + File (logs/)$(NC)"
	@mkdir -p logs certs ca-certs
	@# Auto-detect if running as root/sudo and set ports accordingly
	@if [ "$$(id -u)" -eq 0 ]; then \
		echo "$(GREEN)🔒 Running as root - using privileged ports (80/443)$(NC)"; \
		echo "$(YELLOW)⚠️  Setting use_privileged_ports: true$(NC)"; \
		sed -i.bak 's/use_privileged_ports: false/use_privileged_ports: true/' config.yml 2>/dev/null || true; \
		sed -i.bak 's/use_privileged_ports:false/use_privileged_ports: true/' config.yml 2>/dev/null || true; \
	else \
		echo "$(CYAN)🔗 Running as user - using regular ports (8080/8443)$(NC)"; \
		echo "$(YELLOW)⚠️  Setting use_privileged_ports: false$(NC)"; \
		sed -i.bak 's/use_privileged_ports: true/use_privileged_ports: false/' config.yml 2>/dev/null || true; \
		sed -i.bak 's/use_privileged_ports:true/use_privileged_ports: false/' config.yml 2>/dev/null || true; \
	fi; \
	rm -f config.yml.bak; \
	echo "$(CYAN)🧵 Runtime Mode: Set in config.yml$(NC)"; \
	echo ""; \
	cargo run --bin rust-forward-proxy --release

.PHONY: list-process
list-process: ## List all running proxy processes
	@echo "$(BLUE)Running Rust Forward Proxy Processes:$(NC)"
	@echo "======================================"
	@PROCESSES=$$(ps aux | grep rust-forward-proxy | grep -v grep | grep -v 'make list-process'); \
	if [ -n "$$PROCESSES" ]; then \
		echo "$$PROCESSES" | while read line; do \
			PID=$$(echo "$$line" | awk '{print $$2}'); \
			USER=$$(echo "$$line" | awk '{print $$1}'); \
			CPU=$$(echo "$$line" | awk '{print $$3}'); \
			MEM=$$(echo "$$line" | awk '{print $$4}'); \
			CMD=$$(echo "$$line" | awk '{for(i=11;i<=NF;i++) printf "%s ", $$i; print ""}' | sed 's/ *$$//'); \
			echo "$(GREEN)🔹 PID: $$PID$(NC) | User: $$USER | CPU: $$CPU% | Memory: $$MEM%"; \
			echo "   Command: $$CMD"; \
			echo ""; \
		done; \
		echo "$(CYAN)Total processes: $$(echo "$$PROCESSES" | wc -l | tr -d ' ')$(NC)"; \
		echo "$(CYAN)To stop all: kill \$$(pgrep rust-forward-proxy)$(NC)"; \
	else \
		echo "$(YELLOW)No rust-forward-proxy processes found$(NC)"; \
	fi

.PHONY: build
build: ## Build release binary
	@echo "$(BLUE)Building release binary...$(NC)"
	cargo build --release

.PHONY: build-dev
build-dev: ## Build debug binary
	@echo "$(BLUE)Building debug binary...$(NC)"
	cargo build

# =================================================
# DOCKER COMMANDS
# =================================================

.PHONY: docker-dev
docker-dev: ## Start Docker development environment
	@echo "$(BLUE)Starting Docker development environment...$(NC)"
	$(DOCKER_COMPOSE) up --build

.PHONY: docker-up
docker-up: ## Start Docker environment in background
	@echo "$(BLUE)Starting Docker environment in background...$(NC)"
	$(DOCKER_COMPOSE) up -d --build
	@$(MAKE) docker-status

.PHONY: docker-stop
docker-stop: ## Stop all Docker services
	@echo "$(YELLOW)Stopping Docker services...$(NC)"
	$(DOCKER_COMPOSE) stop

.PHONY: docker-down
docker-down: ## Stop and remove Docker containers
	@echo "$(YELLOW)Stopping and removing Docker containers...$(NC)"
	$(DOCKER_COMPOSE) down

.PHONY: docker-restart
docker-restart: ## Restart Docker services
	@echo "$(BLUE)Restarting Docker services...$(NC)"
	$(DOCKER_COMPOSE) restart
	@$(MAKE) docker-status

.PHONY: docker-logs
docker-logs: ## Show logs from all Docker services
	$(DOCKER_COMPOSE) logs -f

.PHONY: docker-logs-proxy
docker-logs-proxy: ## Show logs from proxy service only
	$(DOCKER_COMPOSE) logs -f rust-proxy

.PHONY: docker-logs-redis
docker-logs-redis: ## Show logs from Redis service only
	$(DOCKER_COMPOSE) logs -f redis

.PHONY: docker-status
docker-status: ## Show Docker service status
	@echo "$(BLUE)Docker Service Status:$(NC)"
	$(DOCKER_COMPOSE) ps

.PHONY: docker-build
docker-build: ## Build Docker containers without starting
	@echo "$(BLUE)Building Docker containers...$(NC)"
	$(DOCKER_COMPOSE) build

.PHONY: docker-shell-proxy
docker-shell-proxy: ## Open shell in proxy container
	$(DOCKER_COMPOSE) exec rust-proxy /bin/sh

.PHONY: docker-shell-redis
docker-shell-redis: ## Open Redis CLI
	$(DOCKER_COMPOSE) exec redis redis-cli

# =================================================
# TESTING COMMANDS
# =================================================

.PHONY: test
test: ## Test local proxy connection (auto-detects port)
	@echo "$(BLUE)Testing local proxy...$(NC)"
	@echo "$(YELLOW)Make sure proxy is running first$(NC)"
	@# Try privileged port first, then regular port
	@if curl -s -f --max-time 5 -x http://127.0.0.1:80 http://httpbin.org/get > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Local proxy test passed (port 80)$(NC)"; \
	elif curl -s -f --max-time 5 -x http://127.0.0.1:8080 http://httpbin.org/get > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Local proxy test passed (port 8080)$(NC)"; \
	else \
		echo "$(RED)❌ Local proxy test failed on both ports 80 and 8080$(NC)"; \
		echo "$(YELLOW)💡 Make sure to run 'make dev' first$(NC)"; \
		exit 1; \
	fi

.PHONY: test-https
test-https: ## Test HTTPS interception (auto-detects port)
	@echo "$(BLUE)Testing HTTPS interception...$(NC)"
	@echo "$(YELLOW)Make sure proxy is running first$(NC)"
	@# Try privileged port first, then regular port
	@if curl -s -f --max-time 5 --proxy-insecure -x http://127.0.0.1:80 https://httpbin.org/get > /dev/null 2>&1; then \
		echo "$(GREEN)✅ HTTPS interception test passed (port 80)$(NC)"; \
	elif curl -s -f --max-time 5 --proxy-insecure -x http://127.0.0.1:8080 https://httpbin.org/get > /dev/null 2>&1; then \
		echo "$(GREEN)✅ HTTPS interception test passed (port 8080)$(NC)"; \
	else \
		echo "$(RED)❌ HTTPS interception test failed on both ports 80 and 8080$(NC)"; \
		exit 1; \
	fi

.PHONY: test-docker
test-docker: ## Test Docker proxy (privileged port 80)
	@echo "$(BLUE)Testing Docker proxy...$(NC)"
	@if curl -s -f --max-time 10 -x http://localhost:80 http://httpbin.org/get > /dev/null; then \
		echo "$(GREEN)✅ Docker proxy test passed (port 80)$(NC)"; \
	else \
		echo "$(RED)❌ Docker proxy test failed$(NC)"; \
		echo "$(YELLOW)💡 Make sure Docker is running with 'make docker-dev'$(NC)"; \
		exit 1; \
	fi

# =================================================
# UTILITIES
# =================================================

.PHONY: clean
clean: ## Clean up Docker resources
	@echo "$(YELLOW)Cleaning up Docker resources...$(NC)"
	$(DOCKER_COMPOSE) down
	docker system prune -f
	@echo "$(GREEN)✅ Cleanup complete$(NC)"

.PHONY: status
status: ## Show application and Docker status
	@echo "$(BLUE)Application Status:$(NC)"
	@echo "$(CYAN)Configuration file: config.yml$(NC)"
	@if [ -f config.yml ]; then \
		echo "$(GREEN)✅ Configuration file exists$(NC)"; \
		echo "$(CYAN)Privileged ports: $$(grep '^use_privileged_ports:' config.yml | awk '{print $$2}')$(NC)"; \
		echo "$(CYAN)Runtime mode: $$(grep '  mode:' config.yml | awk '{print $$2}' | tr -d '"')$(NC)"; \
		echo "$(CYAN)Process count: $$(grep '  process_count:' config.yml | awk '{print $$2}')$(NC)"; \
	else \
		echo "$(RED)❌ Configuration file missing$(NC)"; \
	fi
	@echo ""
	@echo "$(BLUE)Docker Status:$(NC)"
	@$(DOCKER_COMPOSE) ps 2>/dev/null || echo "$(YELLOW)Docker Compose not running$(NC)"
