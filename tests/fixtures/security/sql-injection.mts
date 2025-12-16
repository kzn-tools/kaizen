// Security vulnerability: SQL injection

interface Database {
    execute(query: string): Promise<unknown[]>;
    query(sql: string): Promise<unknown[]>;
    run(statement: string): Promise<void>;
}

function getUserByName(db: Database, username: string): Promise<unknown[]> {
    const query: string = "SELECT * FROM users WHERE name = '" + username + "'";
    return db.execute(query);
}

function searchProducts(db: Database, searchTerm: string): Promise<unknown[]> {
    const sql: string = `SELECT * FROM products WHERE description LIKE '%${searchTerm}%'`;
    return db.query(sql);
}

function deleteRecord(db: Database, id: number): Promise<void> {
    return db.run("DELETE FROM records WHERE id = " + id);
}

async function updateUser(db: Database, userId: number, email: string): Promise<void> {
    const updateQuery: string = "UPDATE users SET email = '" + email + "' WHERE id = " + userId;
    await db.execute(updateQuery);
}

export { getUserByName, searchProducts, deleteRecord, updateUser };
export type { Database };
