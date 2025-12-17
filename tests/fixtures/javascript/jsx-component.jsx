// React JSX component - no errors expected

import React, { useState, useEffect } from 'react';

function Button({ onClick, children, disabled = false }) {
    return (
        <button
            onClick={onClick}
            disabled={disabled}
            className="btn"
        >
            {children}
        </button>
    );
}

const Counter = () => {
    const [count, setCount] = useState(0);

    useEffect(() => {
        document.title = `Count: ${count}`;
    }, [count]);

    return (
        <div className="counter">
            <h1>Count: {count}</h1>
            <Button onClick={() => setCount(count + 1)}>
                Increment
            </Button>
            <Button onClick={() => setCount(0)} disabled={count === 0}>
                Reset
            </Button>
        </div>
    );
};

export { Button };
export default Counter;
