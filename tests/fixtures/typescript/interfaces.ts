// TypeScript interfaces and types - no errors expected

interface User {
    id: number;
    name: string;
    email: string;
    role: UserRole;
    metadata?: Record<string, unknown>;
}

type UserRole = 'admin' | 'user' | 'guest';

interface Repository<T> {
    find(id: number): Promise<T | null>;
    findAll(): Promise<T[]>;
    save(entity: T): Promise<T>;
    delete(id: number): Promise<boolean>;
}

class UserRepository implements Repository<User> {
    private users: Map<number, User> = new Map();

    async find(id: number): Promise<User | null> {
        return this.users.get(id) ?? null;
    }

    async findAll(): Promise<User[]> {
        return Array.from(this.users.values());
    }

    async save(user: User): Promise<User> {
        this.users.set(user.id, user);
        return user;
    }

    async delete(id: number): Promise<boolean> {
        return this.users.delete(id);
    }
}

function createUser(name: string, email: string, role: UserRole = 'user'): User {
    return {
        id: Date.now(),
        name,
        email,
        role,
    };
}

export type { User, UserRole, Repository };
export { UserRepository, createUser };
