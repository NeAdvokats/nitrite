# nitrite

Минимальный пакетный менеджер: собирает пакеты из исходников и ставит их в `~/.local`.

## Установка

Через make:

```sh
git clone https://github.com/NeAdvokats/nitrite
cd nitrite
make
```

Или возьми готовый бинарник из релизов репозитория и перенеси в `/usr/bin`:

```sh
chmod +x nit
sudo mv nit /usr/bin/nit
```

## Использование

```sh
nit --sync              # скачать индекс
nit --install PACKAGE   # установить пакет
nit --find PACKAGE      # ищет конкретный пакет в репозитории
```

`nit` скачивает архив, проверяет `sha256`, распаковывает его, проверяет `PKGINFO`,
запускает `build.sh` и копирует результат в `~/.local`. Временные файлы после
успешной установки удаляются.

`build.sh` получает `SRCDIR` с исходниками и `PKGDIR` для готовых файлов.
Содержимое `PKGDIR` повторяет структуру `~/.local`, например
`$PKGDIR/bin/sifetch` попадёт в `~/.local/bin/sifetch`.

Для сборки пакетов нужны `sh` и инструменты вроде `gcc` и `make`.
