<div align="center">
    <h1><code>plpgsql-analyzer</code></h1>
    <strong>A library to analyze PL/SQL code using WebAssembly.</strong>
    <br />
    <sub>Built with 🦀 by <a href="https://www.cybertec-postgresql.com/en/">CYBERTEC PostgreSQL International GmbH</a></sub>
</div>

## 🗒️️ About

Parse and analyze Oracle PL/SQL code.

## 🛠️ Installation

```bash
npm install --save @cybertec/plpgsql-analyzer
# OR
yarn add @cybertec/plpgsql-analyzer
```

## 🚀 Usage

```typescript
import { analyze, DboType } from "plpgsql-analyzer";

const content = `CREATE FUNCTION my_func
RETURN NUMBER
IS
BEGIN
    RETURN 1;
END my_func;`;

analyze(DboType.Function, content, { tables: {} });
```

## 📝 License

See [LICENSE](./LICENSE).

## 👤 Authors

-   [Christoph Heiss](https://github.com/christoph-heiss)
-   [Kieran Kaelin](https://github.com/KieranKaelin)
-   [Jeremy Sztavinovszki](https://github.com/if-loop69420)
