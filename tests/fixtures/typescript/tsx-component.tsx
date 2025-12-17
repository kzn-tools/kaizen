// TypeScript React component - no errors expected

import React, { useState, useCallback, FC, ReactNode } from 'react';

interface ButtonProps {
    onClick: () => void;
    children: ReactNode;
    variant?: 'primary' | 'secondary' | 'danger';
    disabled?: boolean;
}

const Button: FC<ButtonProps> = ({
    onClick,
    children,
    variant = 'primary',
    disabled = false,
}) => {
    return (
        <button
            onClick={onClick}
            disabled={disabled}
            className={`btn btn-${variant}`}
        >
            {children}
        </button>
    );
};

interface TodoItem {
    id: number;
    text: string;
    completed: boolean;
}

interface TodoListProps {
    initialItems?: TodoItem[];
}

const TodoList: FC<TodoListProps> = ({ initialItems = [] }) => {
    const [items, setItems] = useState<TodoItem[]>(initialItems);
    const [newText, setNewText] = useState<string>('');

    const addItem = useCallback(() => {
        if (!newText.trim()) return;

        const newItem: TodoItem = {
            id: Date.now(),
            text: newText,
            completed: false,
        };

        setItems((prev) => [...prev, newItem]);
        setNewText('');
    }, [newText]);

    const toggleItem = useCallback((id: number) => {
        setItems((prev) =>
            prev.map((item) =>
                item.id === id ? { ...item, completed: !item.completed } : item
            )
        );
    }, []);

    return (
        <div className="todo-list">
            <input
                type="text"
                value={newText}
                onChange={(e) => setNewText(e.target.value)}
                placeholder="Add new item..."
            />
            <Button onClick={addItem}>Add</Button>
            <ul>
                {items.map((item) => (
                    <li
                        key={item.id}
                        onClick={() => toggleItem(item.id)}
                        style={{ textDecoration: item.completed ? 'line-through' : 'none' }}
                    >
                        {item.text}
                    </li>
                ))}
            </ul>
        </div>
    );
};

export type { ButtonProps, TodoItem, TodoListProps };
export { Button, TodoList };
