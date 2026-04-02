# Гра "Козацький бізнес" — Solana Devnet

Цей проєкт реалізує ігрову економіку на блокчейні Solana для конкурсу WhiteBIT.

## Деплой (Devnet Program IDs)
- **Resource Manager:** `4PFu4dsdPuDisbSjUCRjufg8xFysRWNADKViwW43Ch5R`
- **Item NFT:** `AmdVbUTd8VV6XTd6udq3ZgoyqdTAf63iCZythgohzaLG`
- **Search Program:** `BSMa3VZ7xjFrbVBzhv2bGd6PAsiaHYWzwiGnx3xwYfdb`
- **Marketplace:** `BicWhdttM2dX1ENj7GC6kP4JJhbMoynAvjVmfmHzGfwN`
- **Magic Token:** `DusrqDQ7sK5mkSfztr4ZQaagCQz3v1XNSzCYH9uVWuvR`

## Технології
- **Фреймворк:** Anchor 0.29.0
- **Стандарт токенів:** SPL Token-2022
- **NFT:** Metaplex стандарт (реалізовано через PDA authority)
- **Мова:** Rust & TypeScript

## Механіки
1. **Search:** Видобуток ресурсів (WOOD, IRON, LEATHER) з он-чейн таймером 60с.
2. **Crafting:** Спалення ресурсів (CPI) для створення NFT Шаблі або Посоха.
3. **Marketplace:** Продаж NFT, що активує мінт MagicToken (CPI) та спалення предмету.

## Інструкції з налаштування та запуску
1. **Встановлення залежностей:** Переконайтеся, що у вас встановлено Rust, Solana CLI та Anchor. Потім встановіть JS-пакети: `yarn install`
2. **Складання програм:** Скомпілюйте Rust-код та згенеруйте типи (IDL): `anchor build` (якщо виникають проблеми, дивись додатки про проблеми)
3. **Запуск тестів:** Для перевірки логіки в локальній мережі (рекомендовано): `anchor test` ; Для запуску тестів прямо в Devnet (проти вже задеплоєних програм): `anchor test --skip-deploy --provider.cluster devnet`

## Проблеми з білдом
Якщо виникають проблеми з білдом спробуйте застосувати цю послідовність команд (вона опустить всі потрібні залежності до рівня, який може скомпілити 1.75 rust):
- `cargo +1.75.0 update -p unicode-segmentation --precise 1.11.0`
- `cargo +1.75.0 update -p blake3 --precise 1.5.0`
- `cargo +1.75.0 update -p borsh@1.6.1 --precise 1.3.1`
- `cargo +1.75.0 update -p proc-macro-crate@3.5.0 --precise 3.1.0`
- `cargo +1.75.0 update -p indexmap@2.13.0 --precise 2.2.6`

Якщо проблеми з Cargo.lock - видаліть файл і перезберіть заново командою `cargo +1.75.0 generate-lockfile` - потім застосуйте команди вище. Команда `chmod 444 Cargo.lock` заборонить раст аналізатору змінювати динамічно файл (часто піднімає назад версії бібліотек, що ламає білд потім). Для тестів - `sudo chmod 644 Cargo.lock` - назад в звичний стан.

### Приклад, що має бути на солана девнеті:
<img width="1416" height="581" alt="image" src="https://github.com/user-attachments/assets/e55adc49-2b06-452d-8154-da5f1fba1c58" />

