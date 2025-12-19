# no-xss (S002)

Disallow XSS vulnerabilities via untrusted HTML in DOM.

## Description

This rule uses taint analysis to detect when untrusted user input is inserted into the DOM without proper sanitization, which could allow attackers to execute arbitrary JavaScript.

## Rationale

Cross-Site Scripting (XSS) allows attackers to:
- Steal user session cookies
- Capture user credentials
- Redirect users to malicious sites
- Deface websites
- Perform actions as the victim

## Examples

### Bad

```javascript
function displayMessage(req, res) {
    const message = req.query.message;
    element.innerHTML = message;  // XSS vulnerability
}

function render(req, res) {
    const name = req.body.name;
    document.write(`<h1>Hello ${name}</h1>`);  // XSS vulnerability
}

function update(req, res) {
    const html = req.body.content;
    el.outerHTML = html;  // XSS vulnerability
}
```

### Good

```javascript
function displayMessage(req, res) {
    const message = req.query.message;
    element.textContent = message;  // Safe - no HTML parsing
}

function render(req, res) {
    const name = req.body.name;
    const el = document.createElement('h1');
    el.textContent = `Hello ${name}`;  // Safe
    container.appendChild(el);
}

function update(req, res) {
    const html = req.body.content;
    element.innerHTML = DOMPurify.sanitize(html);  // Sanitized
}
```

## Taint Sources

The rule tracks data from:
- `req.body.*` - Request body
- `req.query.*` - Query parameters
- `req.params.*` - URL parameters
- User input from various sources

## Taint Sinks

DOM methods that parse HTML:
- `element.innerHTML`
- `element.outerHTML`
- `document.write()`
- `document.writeln()`
- `insertAdjacentHTML()`

## Prevention

1. **Use textContent** instead of innerHTML when displaying text
2. **Use DOM APIs** to create elements programmatically
3. **Sanitize HTML** with DOMPurify or similar libraries
4. **Use Content Security Policy (CSP)** as defense in depth
5. **Use a framework** that auto-escapes (React, Vue, Angular)

## Framework Safety

Modern frameworks provide auto-escaping:

```jsx
// React - automatically escapes
function Message({ text }) {
    return <div>{text}</div>;  // Safe - auto-escaped
}

// Dangerous - opt-in to unsafe behavior
function Message({ html }) {
    return <div dangerouslySetInnerHTML={{ __html: html }} />;  // Flagged
}
```

## Configuration

### Disable the rule

```toml
[rules]
disabled = ["S002"]
# or
disabled = ["no-xss"]
```

## When Not To Use It

This rule should always be enabled for web applications.

## Related Rules

- [no-sql-injection](no-sql-injection.md) - SQL injection
- [no-eval-injection](no-eval-injection.md) - Code injection
