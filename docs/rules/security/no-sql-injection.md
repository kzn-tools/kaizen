# no-sql-injection (S001)

Disallow SQL injection vulnerabilities via untrusted data.

## Description

This rule uses taint analysis to detect when untrusted user input flows into SQL queries, which could allow attackers to execute arbitrary SQL commands.

## Rationale

SQL injection is one of the most critical security vulnerabilities:
- Attackers can read, modify, or delete database data
- Attackers can bypass authentication
- Attackers can escalate privileges
- Can lead to complete system compromise

## Examples

### Bad

```javascript
function handler(req, res) {
    const userId = req.body.userId;
    db.query("SELECT * FROM users WHERE id = " + userId);  // SQL injection
}

function search(req, res) {
    const term = req.query.term;
    db.query(`SELECT * FROM products WHERE name LIKE '%${term}%'`);  // SQL injection
}
```

### Good

```javascript
function handler(req, res) {
    const userId = req.body.userId;
    db.query("SELECT * FROM users WHERE id = ?", [userId]);  // Parameterized
}

function search(req, res) {
    const term = req.query.term;
    db.query("SELECT * FROM products WHERE name LIKE ?", [`%${term}%`]);  // Parameterized
}

// Using an ORM
const user = await User.findById(req.body.userId);
```

## Taint Sources

The rule tracks data from:
- `req.body.*` - Request body
- `req.query.*` - Query parameters
- `req.params.*` - URL parameters
- `req.headers.*` - Request headers
- `process.env.*` - Environment variables
- `process.argv` - Command line arguments

## Taint Sinks

SQL-related functions:
- `db.query()`
- `connection.query()`
- `pool.query()`
- `knex.raw()`
- `sequelize.query()`
- And other SQL execution methods

## Prevention

1. **Use parameterized queries** (prepared statements)
2. **Use an ORM** (Sequelize, Prisma, TypeORM)
3. **Use query builders** (Knex.js)
4. **Validate and sanitize input** (as defense in depth)

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["S001"]
# or
disabled = ["no-sql-injection"]
```

## When Not To Use It

This rule should always be enabled for database-connected applications.

## Related Rules

- [no-command-injection](no-command-injection.md) - Shell command injection
- [no-eval-injection](no-eval-injection.md) - Code execution injection
- [no-xss](no-xss.md) - Cross-site scripting
