#!/bin/bash

echo "🧪 Тестирование всех типов метаданных"
echo ""

# Проверка наличия файла
if [ ! -f "photo.jpg" ]; then
    echo "❌ Файл photo.jpg не найден"
    echo "Создайте тестовый файл или укажите другое имя"
    exit 1
fi

echo "📋 1. Health check:"
curl -s http://localhost:3000/health | jq .
echo ""

echo "📋 2. Чтение ВСЕХ метаданных:"
curl -s -X POST -F "file=@photo.jpg" http://localhost:3000/read | jq '.data.metadata | keys | length'
echo ""

echo "📋 3. Чтение только EXIF:"
curl -s -X POST -F "file=@photo.jpg" -F "type=exif" http://localhost:3000/read | jq '.data.metadata | keys | length'
echo ""

echo "📋 4. Чтение только GPS:"
curl -s -X POST -F "file=@photo.jpg" -F "type=gps" http://localhost:3000/read | jq '.data.metadata | keys'
echo ""

echo "📋 5. Чтение только C2PA (AI-подписи):"
curl -s -X POST -F "file=@photo.jpg" -F "type=c2pa" http://localhost:3000/read | jq '.data.metadata | keys'
echo ""

echo "📋 6. Запись метаданных (Make, Model, Artist):"
curl -s -X POST \
  -F "file=@photo.jpg" \
  -F 'metadata={"Make":"Nikon","Model":"Z9","Artist":"Test User"}' \
  http://localhost:3000/write \
  --output modified.jpg
echo "✅ Файл сохранён: modified.jpg"
echo ""

echo "📋 7. Проверка изменений:"
curl -s -X POST -F "file=@modified.jpg" http://localhost:3000/read | jq '.data.metadata | {Make, Model, Artist}'
echo ""

echo "📋 8. Удаление только C2PA (если есть):"
curl -s -X POST -F "file=@photo.jpg" -F "type=c2pa" http://localhost:3000/delete --output no_c2pa.jpg
echo "✅ Файл сохранён: no_c2pa.jpg"
echo ""

echo "📋 9. Удаление всех метаданных:"
curl -s -X POST -F "file=@photo.jpg" http://localhost:3000/delete --output clean.jpg
echo "✅ Файл сохранён: clean.jpg"
echo ""

echo "📋 10. Проверка чистого файла (метаданные должны отсутствовать):"
curl -s -X POST -F "file=@clean.jpg" http://localhost:3000/read | jq '.data.metadata | keys | length'
echo ""

echo "✅ Тестирование завершено!"
echo ""
echo "📁 Созданные файлы:"
ls -lh modified.jpg no_c2pa.jpg clean.jpg 2>/dev/null || echo "  (файлы не найдены)"
