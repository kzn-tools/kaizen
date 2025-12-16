// Security vulnerability: hardcoded secrets

const API_KEY: string = "sk-1234567890abcdef1234567890abcdef";
const DATABASE_PASSWORD: string = "super_secret_password_123";
const JWT_SECRET: string = "my-jwt-secret-key-do-not-share";

interface Config {
    apiKey: string;
    dbPassword: string;
    jwtSecret: string;
}

const config: Config = {
    apiKey: "AKIAIOSFODNN7EXAMPLE",
    dbPassword: "admin123",
    jwtSecret: "secret123"
};

async function connectToApi(): Promise<Response> {
    return fetch("https://api.example.com/data", {
        headers: {
            "Authorization": "Bearer " + API_KEY,
            "X-API-Key": "abcd1234efgh5678ijkl9012mnop3456"
        }
    });
}

function getDatabaseConnection(): string {
    return `postgresql://admin:${DATABASE_PASSWORD}@localhost:5432/mydb`;
}

export { config, connectToApi, getDatabaseConnection };
