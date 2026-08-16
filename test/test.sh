#!/bin/bash

# test.sh - Полное тестирование EXIF API
# Использование: ./test.sh

set -e

# Цвета для вывода
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Проверка наличия jq
if ! command -v jq &> /dev/null; then
    echo -e "${RED}❌ jq не установлен. Установите: sudo apt install jq${NC}"
    exit 1
fi

# Проверка наличия тестового файла
TEST_FILE="photo.jpg"
if [ ! -f "$TEST_FILE" ]; then
    echo -e "${RED}❌ Файл $TEST_FILE не найден в текущей директории${NC}"
    echo "Создайте тестовый файл или укажите другое имя"
    exit 1
fi

# Проверка доступности сервера
echo -e "${BLUE}🔍 Проверка доступности сервера...${NC}"
if ! curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo -e "${RED}❌ Сервер не доступен на http://localhost:3000${NC}"
    echo "Запустите: docker compose up -d"
    exit 1
fi
echo -e "${GREEN}✅ Сервер доступен${NC}"
echo ""

# Функция для разделения
separator() {
    echo ""
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# Функция для проверки успешности
check_success() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ УСПЕШНО${NC}"
    else
        echo -e "${RED}❌ ОШИБКА (код: $1)${NC}"
    fi
}

# Создание папки для результатов
mkdir -p test_results

# 1. HEALTH CHECK
separator "1. HEALTH CHECK"
curl -s http://localhost:3000/health | jq .
echo ""

# 2. READ ALL METADATA (ИСХОДНЫЙ)
separator "2. READ ALL METADATA (Исходный файл)"
echo "Чтение всех метаданных из исходного файла..."
TAG_COUNT=$(curl -s -X POST -F "file=@$TEST_FILE" http://localhost:3000/read | jq '.data.tag_count')
echo -e "Количество тегов: ${GREEN}$TAG_COUNT${NC}"
echo ""

# 3. READ EXIF (ИСХОДНЫЙ)
separator "3. READ EXIF (Исходный файл)"
echo "Чтение только EXIF из исходного файла..."
curl -s -X POST -F "file=@$TEST_FILE" -F "type=exif" http://localhost:3000/read | jq '.data.metadata | keys | length'
echo ""

# 4. READ GPS (ИСХОДНЫЙ)
separator "4. READ GPS (Исходный файл)"
echo "Чтение только GPS из исходного файла..."
curl -s -X POST -F "file=@$TEST_FILE" -F "type=gps" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 5. WRITE IPTC
separator "5. WRITE IPTC"
echo "Запись IPTC тегов..."
curl -s -X POST \
  -F "file=@$TEST_FILE" \
  -F 'metadata={"Keywords":"test,photo,exif,api","Caption":"My beautiful photo","City":"Moscow","Country":"Russia","Credit":"Test Studio"}' \
  -F "type=iptc" \
  http://localhost:3000/write \
  --output test_results/test_iptc.jpg
check_success $?
echo ""

# 6. READ IPTC (ПОСЛЕ ЗАПИСИ)
separator "6. READ IPTC (После записи)"
echo "Чтение только IPTC из записанного файла..."
curl -s -X POST -F "file=@test_results/test_iptc.jpg" -F "type=iptc" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 7. WRITE XMP
separator "7. WRITE XMP"
echo "Запись XMP тегов..."
curl -s -X POST \
  -F "file=@$TEST_FILE" \
  -F 'metadata={"Creator":"John Doe","Rights":"Copyright 2024","Description":"Test XMP metadata","Subject":"Photography","Title":"Test Image"}' \
  -F "type=xmp" \
  http://localhost:3000/write \
  --output test_results/test_xmp.jpg
check_success $?
echo ""

# 8. READ XMP (ПОСЛЕ ЗАПИСИ)
separator "8. READ XMP (После записи)"
echo "Чтение только XMP из записанного файла..."
curl -s -X POST -F "file=@test_results/test_xmp.jpg" -F "type=xmp" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 9. WRITE C2PA
separator "9. WRITE C2PA"
echo "Запись C2PA (AI-подписи)..."
echo -e "${YELLOW}⚠️ C2PA — это обычно подпись, которую ставят инструменты ИИ.${NC}"
echo -e "${YELLOW}   Мы можем записать любые метаданные как C2PA.${NC}"
curl -s -X POST \
  -F "file=@$TEST_FILE" \
  -F 'metadata={"Creator":"AI Generator","Model":"Stable Diffusion","Version":"3.0","Generator":"ComfyUI"}' \
  -F "type=c2pa" \
  http://localhost:3000/write \
  --output test_results/test_c2pa.jpg
check_success $?
echo ""

# 10. READ C2PA (ПОСЛЕ ЗАПИСИ)
separator "10. READ C2PA (После записи)"
echo "Чтение только C2PA из записанного файла..."
curl -s -X POST -F "file=@test_results/test_c2pa.jpg" -F "type=c2pa" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 11. WRITE MAKERNOTES
separator "11. WRITE MAKERNOTES"
echo "Запись MakerNotes (данные производителя)..."
echo -e "${YELLOW}⚠️ MakerNotes — это специфичные данные производителя.${NC}"
echo -e "${YELLOW}   Не все теги можно записать, но некоторые поддерживаются.${NC}"
curl -s -X POST \
  -F "file=@$TEST_FILE" \
  -F 'metadata={"SerialNumber":"1234567890","FirmwareVersion":"2.0.0","CameraSettings":"Custom"}' \
  -F "type=makernotes" \
  http://localhost:3000/write \
  --output test_results/test_makernotes.jpg
check_success $?
echo ""

# 12. READ MAKERNOTES (ПОСЛЕ ЗАПИСИ)
separator "12. READ MAKERNOTES (После записи)"
echo "Чтение только MakerNotes из записанного файла..."
curl -s -X POST -F "file=@test_results/test_makernotes.jpg" -F "type=makernotes" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 13. WRITE GPS
separator "13. WRITE GPS"
echo "Запись GPS координат (Москва, Красная площадь)..."
curl -s -X POST \
  -F "file=@$TEST_FILE" \
  -F 'metadata={"GPSLatitude":"55.753215","GPSLongitude":"37.620393","GPSLatitudeRef":"N","GPSLongitudeRef":"E","GPSAltitude":"156"}' \
  -F "type=gps" \
  http://localhost:3000/write \
  --output test_results/test_gps.jpg
check_success $?
echo ""

# 14. READ GPS (ПОСЛЕ ЗАПИСИ)
separator "14. READ GPS (После записи)"
echo "Чтение только GPS из записанного файла..."
curl -s -X POST -F "file=@test_results/test_gps.jpg" -F "type=gps" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 15. WRITE ALL
separator "15. WRITE ALL"
echo "Запись всех типов метаданных без фильтра..."
curl -s -X POST \
  -F "file=@$TEST_FILE" \
  -F 'metadata={"Make":"Canon","Model":"EOS R5","Artist":"Pro Photographer","GPSLatitude":"55.753215","GPSLongitude":"37.620393","Keywords":"test,photo,exif","Creator":"John Doe"}' \
  http://localhost:3000/write \
  --output test_results/test_all.jpg
check_success $?
echo ""

# 16. READ ALL (ПОСЛЕ ЗАПИСИ ВСЕГО)
separator "16. READ ALL (После записи всего)"
echo "Чтение всех метаданных из файла со всеми записанными тегами..."
TAG_COUNT_ALL=$(curl -s -X POST -F "file=@test_results/test_all.jpg" http://localhost:3000/read | jq '.data.tag_count')
echo -e "Количество тегов: ${GREEN}$TAG_COUNT_ALL${NC}"
echo ""

# 17. DELETE SPECIFIC EXIF TAGS
separator "17. DELETE SPECIFIC EXIF TAGS"
echo "Удаление конкретных EXIF тегов (Make, Model)..."
curl -s -X POST \
  -F "file=@$TEST_FILE" \
  -F 'tags=["Make","Model"]' \
  -F "type=exif" \
  http://localhost:3000/delete \
  --output test_results/delete_exif_tags.jpg
check_success $?
echo "Проверка: теги должны отсутствовать..."
curl -s -X POST -F "file=@test_results/delete_exif_tags.jpg" -F "type=exif" http://localhost:3000/read | jq '.data.metadata | {Make, Model}'
echo ""

# 18. DELETE ALL GPS
separator "18. DELETE ALL GPS"
echo "Удаление всех GPS тегов..."
curl -s -X POST -F "file=@$TEST_FILE" -F "type=gps" http://localhost:3000/delete --output test_results/delete_gps.jpg
check_success $?
echo "Проверка: GPS теги должны отсутствовать..."
curl -s -X POST -F "file=@test_results/delete_gps.jpg" -F "type=gps" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 19. DELETE ALL IPTC
separator "19. DELETE ALL IPTC"
echo "Удаление всех IPTC тегов..."
curl -s -X POST -F "file=@test_results/test_iptc.jpg" -F "type=iptc" http://localhost:3000/delete --output test_results/delete_iptc.jpg
check_success $?
echo "Проверка: IPTC теги должны отсутствовать..."
curl -s -X POST -F "file=@test_results/delete_iptc.jpg" -F "type=iptc" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 20. DELETE ALL XMP
separator "20. DELETE ALL XMP"
echo "Удаление всех XMP тегов..."
curl -s -X POST -F "file=@test_results/test_xmp.jpg" -F "type=xmp" http://localhost:3000/delete --output test_results/delete_xmp.jpg
check_success $?
echo "Проверка: XMP теги должны отсутствовать..."
curl -s -X POST -F "file=@test_results/delete_xmp.jpg" -F "type=xmp" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 21. DELETE ALL C2PA
separator "21. DELETE ALL C2PA"
echo "Удаление всех C2PA (AI-подписи)..."
curl -s -X POST -F "file=@test_results/test_c2pa.jpg" -F "type=c2pa" http://localhost:3000/delete --output test_results/delete_c2pa.jpg
check_success $?
echo "Проверка: C2PA теги должны отсутствовать..."
curl -s -X POST -F "file=@test_results/delete_c2pa.jpg" -F "type=c2pa" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 22. DELETE ALL MAKERNOTES
separator "22. DELETE ALL MAKERNOTES"
echo "Удаление всех MakerNotes..."
curl -s -X POST -F "file=@test_results/test_makernotes.jpg" -F "type=makernotes" http://localhost:3000/delete --output test_results/delete_makernotes.jpg
check_success $?
echo "Проверка: MakerNotes должны отсутствовать..."
curl -s -X POST -F "file=@test_results/delete_makernotes.jpg" -F "type=makernotes" http://localhost:3000/read | jq '.data.metadata'
echo ""

# 23. DELETE ALL METADATA
separator "23. DELETE ALL METADATA"
echo "Удаление ВСЕХ метаданных (полная очистка)..."
curl -s -X POST -F "file=@$TEST_FILE" http://localhost:3000/delete --output test_results/clean_all.jpg
check_success $?
echo "Проверка: все метаданные должны отсутствовать..."
TAG_COUNT_CLEAN=$(curl -s -X POST -F "file=@test_results/clean_all.jpg" http://localhost:3000/read | jq '.data.tag_count')
echo -e "Количество тегов после полной очистки: ${GREEN}$TAG_COUNT_CLEAN${NC}"
if [ "$TAG_COUNT_CLEAN" -eq 0 ]; then
    echo -e "${GREEN}✅ Все метаданные успешно удалены!${NC}"
else
    echo -e "${RED}⚠️ Осталось $TAG_COUNT_CLEAN тегов${NC}"
fi
echo ""

# 24. COMPARE FILE SIZES
separator "24. COMPARE FILE SIZES"
echo "Сравнение размеров файлов:"
echo ""
echo -e "${BLUE}Исходный файл:${NC}"
ls -lh $TEST_FILE
echo ""
echo -e "${BLUE}Результаты тестов:${NC}"
ls -lh test_results/*.jpg 2>/dev/null || echo "  (файлы не найдены)"
echo ""

# 25. SUMMARY
separator "25. SUMMARY"
echo -e "${GREEN}✅ Тестирование завершено!${NC}"
echo ""
echo -e "${BLUE}📁 Результаты сохранены в папке: test_results/${NC}"
echo ""
echo -e "${BLUE}📋 Список созданных файлов:${NC}"
ls -1 test_results/*.jpg 2>/dev/null | sed 's/^/  - /'
echo ""
echo -e "${BLUE}📊 Статистика:${NC}"
echo "  - Исходный файл: $(ls -lh $TEST_FILE | awk '{print $5}')"
if [ -f "test_results/test_exif.jpg" ]; then
    echo "  - С EXIF: $(ls -lh test_results/test_exif.jpg | awk '{print $5}')"
fi
if [ -f "test_results/test_gps.jpg" ]; then
    echo "  - С GPS: $(ls -lh test_results/test_gps.jpg | awk '{print $5}')"
fi
if [ -f "test_results/test_iptc.jpg" ]; then
    echo "  - С IPTC: $(ls -lh test_results/test_iptc.jpg | awk '{print $5}')"
fi
if [ -f "test_results/test_xmp.jpg" ]; then
    echo "  - С XMP: $(ls -lh test_results/test_xmp.jpg | awk '{print $5}')"
fi
if [ -f "test_results/test_c2pa.jpg" ]; then
    echo "  - С C2PA: $(ls -lh test_results/test_c2pa.jpg | awk '{print $5}')"
fi
if [ -f "test_results/test_makernotes.jpg" ]; then
    echo "  - С MakerNotes: $(ls -lh test_results/test_makernotes.jpg | awk '{print $5}')"
fi
if [ -f "test_results/test_all.jpg" ]; then
    echo "  - Со всем: $(ls -lh test_results/test_all.jpg | awk '{print $5}')"
fi
if [ -f "test_results/clean_all.jpg" ]; then
    echo "  - Без метаданных: $(ls -lh test_results/clean_all.jpg | awk '{print $5}')"
fi
echo ""
echo -e "${YELLOW}📖 Пример проверки конкретного файла:${NC}"
echo "  curl -X POST -F 'file=@test_results/test_exif.jpg' http://localhost:3000/read | jq ."
echo ""
echo -e "${YELLOW}🗑️ Очистка тестовых файлов:${NC}"
echo "  rm -rf test_results/"
