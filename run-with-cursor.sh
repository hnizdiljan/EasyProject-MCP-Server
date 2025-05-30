#!/bin/bash

# EasyProject MCP Server - Script pro spuštění s Cursor
# Použití: ./run-with-cursor.sh [API_KEY] [BASE_URL]

set -e

# Barvy pro výstup
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Parametry
API_KEY=${1:-$EASYPROJECT_API_KEY}
BASE_URL=${2:-$EASYPROJECT_BASE_URL}
LOG_LEVEL=${3:-"info"}

echo -e "${GREEN}🚀 EasyProject MCP Server Setup${NC}"
echo -e "${GREEN}================================${NC}"

# Kontrola Rust instalace
echo -e "${YELLOW}📋 Kontroluji Rust instalaci...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust není nainstalován!${NC}"
    echo -e "${RED}   Nainstalujte Rust z https://rustup.rs/${NC}"
    echo -e "${RED}   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Rust je nainstalován${NC}"

# Kontrola API klíče
if [ -z "$API_KEY" ]; then
    echo -e "${RED}❌ Chybí EASYPROJECT_API_KEY!${NC}"
    echo -e "${RED}   Nastavte environment proměnnou nebo předejte jako parametr${NC}"
    echo -e "${RED}   Příklad: ./run-with-cursor.sh 'your-key' 'https://your-instance.com'${NC}"
    exit 1
fi

if [ -z "$BASE_URL" ]; then
    echo -e "${RED}❌ Chybí EASYPROJECT_BASE_URL!${NC}"
    echo -e "${RED}   Nastavte environment proměnnou nebo předejte jako parametr${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Konfigurace je v pořádku${NC}"

# Kompilace
echo -e "${YELLOW}🔨 Kompiluji MCP server...${NC}"
cargo build --release

echo -e "${GREEN}✅ Kompilace dokončena${NC}"

# Zobrazení konfigurace pro Cursor
echo ""
echo -e "${CYAN}📋 Konfigurace pro Cursor:${NC}"
echo -e "${CYAN}=========================${NC}"

CURRENT_PATH=$(pwd)
EXECUTABLE_PATH="$CURRENT_PATH/target/release/easyproject-mcp-server"

# Vytvoření JSON konfigurace
CONFIG_JSON=$(cat <<EOF
{
  "mcpServers": {
    "easyproject": {
      "command": "$EXECUTABLE_PATH",
      "args": [],
      "env": {
        "EASYPROJECT_API_KEY": "$API_KEY",
        "EASYPROJECT_BASE_URL": "$BASE_URL",
        "MCP_LOG_LEVEL": "$LOG_LEVEL"
      }
    }
  }
}
EOF
)

echo -e "${WHITE}$CONFIG_JSON${NC}"

# Uložení konfigurace do souboru
CONFIG_FILE="cursor-mcp-config.json"
echo "$CONFIG_JSON" > "$CONFIG_FILE"
echo ""
echo -e "${GREEN}✅ Konfigurace uložena do $CONFIG_FILE${NC}"

# Nastavení práv
chmod +x "$EXECUTABLE_PATH"

# Instrukce pro Cursor
echo ""
echo -e "${CYAN}🎯 Další kroky:${NC}"
echo -e "${CYAN}==============${NC}"
echo -e "${WHITE}1. Otevřete Cursor nastavení (Cmd/Ctrl + ,)${NC}"
echo -e "${WHITE}2. Najděte sekci 'MCP Servers'${NC}"
echo -e "${WHITE}3. Zkopírujte výše uvedenou konfiguraci${NC}"
echo -e "${WHITE}4. Nebo použijte soubor: $CONFIG_FILE${NC}"
echo -e "${WHITE}5. Restartujte Cursor${NC}"

# Test spuštění
echo ""
echo -e "${YELLOW}🧪 Testování serveru...${NC}"

export EASYPROJECT_API_KEY="$API_KEY"
export EASYPROJECT_BASE_URL="$BASE_URL"
export MCP_LOG_LEVEL="$LOG_LEVEL"

echo -e "${YELLOW}Spouštím server v test módu...${NC}"
echo -e "${YELLOW}Pro ukončení použijte Ctrl+C${NC}"
echo ""

"$EXECUTABLE_PATH" 