# AnarchyCore

![Rust](https://img.shields.io/badge/Rust-red?style=for-the-badge&logo=rust)
![Axum](https://img.shields.io/badge/Axum-000000?style=for-the-badge&logo=axum)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-316192?style=for-the-badge&logo=postgresql&logoColor=white)

Простой многопользовательский 2D игровой сервер реального времени, разработанный на Rust с использованием фреймворка Axum. Проект демонстрирует реализацию аутентификации пользователя (JWT), управление состоянием игроков через WebSocket и сохранение данных в базе данных PostgreSQL.

## 🚀 Возможности

* **Аутентификация Пользователей:** Регистрация и вход с использованием JWT-токенов для безопасного доступа.
* **WebSocket-коммуникация:** Обмен данными о позициях игроков в реальном времени между сервером и клиентами.
* **Сохранение Состояния Игроков:** Позиции игроков и статус "онлайн" сохраняются в базе данных PostgreSQL.
* **Игровое Поле:** Базовое игровое поле, где игроки могут перемещаться.
* **Поддержка Многих Игроков:** Отображение позиций других подключенных игроков на карте.

## 🛠️ Технологии

* **Axum:** Веб-фреймворк для построения API и обработки WebSocket-соединений.
* **Tokio:** Асинхронный рантайм для Rust.
* **SQLx:** Асинхронный ORM для взаимодействия с базой данных PostgreSQL.
* **Bcrypt:** Для хеширования паролей пользователей.
* **Jsonwebtoken:** Для работы с JWT-токенами.
* **Chrono:** Для работы со временем (например, для срока действия JWT).
* **Dotenv:** Для загрузки переменных окружения из файла `.env`.

## 📦 Установка и Запуск

### 1. Подготовка Базы Данных PostgreSQL

Убедитесь, что у вас установлен и запущен PostgreSQL. Создайте новую базу данных для проекта, например:

```bash
psql -U postgres
CREATE DATABASE anarchy_core;
\q
````

### 2\. Настройка Сервера

1.  **Клонируйте репозиторий:**
    ```bash
    git clone [https://github.com/alex-pyslar/anarchy_core.git](https://github.com/alex-pyslar/anarchy_core.git)
    ```
2.  **Создайте файл `.env`:** В корневой директории вашего Rust-проекта (там же, где `Cargo.toml`) создайте файл с именем `.env` и добавьте в него следующие переменные окружения:
    ```env
    DATABASE_URL=postgres://postgres:ваш_пароль@localhost:5432/anarchy_core
    JWT_SECRET=ваш_очень_секретный_ключ_для_jwt_токенов
    ```
    *Замените `ваш_пароль` на пароль вашего пользователя PostgreSQL и `ваш_очень_секретный_ключ_для_jwt_токенов` на любую длинную случайную строку.*
3.  **Выполните миграцию базы данных:**
    ```sql
    CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        login VARCHAR(255) UNIQUE NOT NULL,
        hashed_password VARCHAR(255) NOT NULL
    );

    CREATE TABLE players (
        user_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
        x DOUBLE PRECISION NOT NULL DEFAULT 0.0,
        y DOUBLE PRECISION NOT NULL DEFAULT 0.0,
        z DOUBLE PRECISION NOT NULL DEFAULT 0.0,
        is_online BOOLEAN NOT NULL DEFAULT FALSE
    );
    ```
4.  **Соберите и запустите сервер:**
    ```bash
    cargo build
    cargo run
    ```
    Сервер будет запущен на `http://127.0.0.1:3000`.

## 🤝 Вклад

Приветствуются любые вклады, предложения и исправления ошибок\! Пожалуйста, откройте Issue или Pull Request.