# 🚀 Core Principles for React TypeScript Projects

This document outlines the core coding standards, conventions, and best practices for React TypeScript development. Use this as a template and foundation for React projects.

---

## Table of Contents

1. [File and Folder Naming](#1-file-and-folder-naming)
2. [Import Organization](#2-import-organization)
3. [TypeScript Types](#3-typescript-types)
4. [React Components](#4-react-components)
5. [Hooks](#5-hooks)
6. [State Management (Redux)](#6-state-management-redux)
7. [Styling](#7-styling)
8. [Testing](#8-testing)
9. [Code Organization](#9-code-organization)
10. [Best Practices](#10-best-practices)
11. [Team Workflow](#11-team-workflow)

---

## 1. File and Folder Naming

### Components
- **PascalCase** for React component files and folders
  - ✅ `Button.tsx`, `LayoutRoot.tsx`, `UserProfile/`
  - ❌ `button.tsx`, `layout-root.tsx`
- Each component in its own file
- Subcomponents in a folder named after the parent component
- Re-export via `index.ts` in component folders

### Utilities, Hooks, and Helpers
- **kebab-case** for utility files, hooks, and helpers
  - ✅ `use-debounce.ts`, `filter-context.ts`, `add-cache-version.ts`
  - ❌ `useDebounce.ts`, `filterContext.ts`

### Models/Types
- **PascalCase** for model files
  - ✅ `User.ts`, `Client.ts`, `Proposal.ts`
- Shared types live in `types/` folder

### Redux Slices
- **camelCase** for feature folders
  - ✅ `auth/`, `editor/`, `interaction/`
- Standard slice file structure:
  - `types.ts` - Type definitions
  - `selectors.ts` - Memoized selectors
  - `actions.ts` - Extended actions (async operations)
  - `index.ts` - Slice definition and exports

### Tests
- Match component name with `.test.tsx` suffix
  - ✅ `Button.test.tsx`, `LayoutRoot.test.tsx`

### Branch Naming
- Use `TASKID-description` format (e.g., `M3D-100-addAuthFlow`)

---

## 2. Import Organization

### Import Order
1. External dependencies (React, third-party libraries)
2. Internal path aliases (`@app/*`)
3. Relative imports
4. Type-only imports (use `import type`)

### Path Aliases
Use TypeScript path aliases for internal imports:

```typescript
import { Button, Modal } from '@app/components';
import { useDebounce } from '@app/hooks';
import { User } from '@app/models';
import { fetcher } from '@app/utils';
import { authActions } from '@app/store';
```

Available aliases (configure in `tsconfig.json` and `vite.config.ts`):
- `@app/components` - All components
- `@app/utils` - Utility functions
- `@app/pages` - Page components
- `@app/store` - Redux store, actions, selectors
- `@app/models` - Type definitions and models
- `@app/services` - Service layer
- `@app/helpers` - Helper functions
- `@app/hooks` - Custom hooks

### Named Imports
Always use named imports from React:

```typescript
import { useState, useEffect, type ReactNode } from 'react';
import { forwardRef } from 'react';
```

Avoid default React import:
```typescript
import React from 'react';  // ❌ Bad
```

### Object Destructuring
Always destructure imports:

```typescript
// ❌ Bad
import React from 'react';
React.forwardRef(() => {})

// ✅ Good
import { forwardRef } from 'react';
forwardRef(() => {})
```

---

## 3. TypeScript Types

### Type vs Interface
- **Prefer `type`** for most cases
- Use `interface` only when you need declaration merging or `implements`

**When to Use `type`:**
- Unions and intersections (e.g., `type Status = "open" | "closed"`)
- Composing multiple types together (e.g., `type UserWithRole = User & Role`)
- Function signatures (e.g., `type ClickHandler = (e: MouseEvent) => void`)
- Aliasing primitives, tuples, or other types

**When to Use `interface`:**
- Defining the shape of objects, especially for public APIs or when you expect extension via declaration merging

```typescript
// Prefer type for unions and intersections
type User = {
  id: string;
  name: string;
  email: string;
};

type Status = 'pending' | 'active' | 'inactive';
type ClickHandler = (e: MouseEvent) => void;
type UserWithRole = User & Role;
```

### Props Typing
- **Local component props**: Use `type Props = {...}`
- **Shared/exported props**: Use `ComponentNameProps`

```typescript
type Props = {
  label?: string;
  value: string;
  onChange: (value: string) => void;
};

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: 'primary' | 'secondary';
  isLoading?: boolean;
};
```

### Boolean Naming
Always prefix boolean properties with `is` or `has`:

```typescript
type Props = {
  isOpen: boolean;
  hasPermission: boolean;
  isLoading: boolean;
  isDisabled?: boolean;
};
```

### Type-Only Imports
Use `import type` for type-only imports:

```typescript
import type { User } from '@app/models';
import type { ReactNode } from 'react';
import type { ButtonHTMLAttributes } from 'react';
```

### Avoid Interface Prefix
❌ Don't use `I` prefix for interfaces/types:
```typescript
type IUser = { ... }  // ❌ Bad
type User = { ... }    // ✅ Good
```

### Extending HTML Attributes
Extend native HTML element types when needed:

```typescript
type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: 'primary' | 'secondary';
  isLoading?: boolean;
};

type InputProps = InputHTMLAttributes<HTMLInputElement> & {
  label?: string;
};
```

### Type Safety
- Avoid `any` type (disabled in tests only)
- Use proper type annotations
- Leverage TypeScript's type inference where appropriate

---

## 4. React Components

### Component Declaration
Always use `const` with arrow functions:

```typescript
export const Button = ({ children, onClick }: Props) => {
  return <button onClick={onClick}>{children}</button>;
};
```

### Forwarding Refs
Use named import `forwardRef` for components that need refs:

```typescript
import { forwardRef } from 'react';

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, children, ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(buttonVariants({ variant }), className)}
        {...props}
      >
        {children}
      </button>
    );
  },
);

Button.displayName = 'Button';
```

### Props Destructuring
Always destructure props in the function signature:

```typescript
export const Input = ({ className, label, id, type = 'text', ...props }: Props) => {
  return (
    <>
      {label && <label htmlFor={id}>{label}</label>}
      <input id={id} type={type} className={cn('base-styles', className)} {...props} />
    </>
  );
};
```

### Default Props
Assign defaults in function parameters:

```typescript
export const Modal = ({ isOpen, onClose, children, size = 'default' }: Props) => {
  // ...
};

const MyComponent = ({ name = '', count = 0 }: Props) => {
  // ...
};
```

### Conditional Rendering
- Use **ternary operators** for conditional rendering
- **Never use `if` statements** inside JSX return
- For mutually exclusive branches, use single ternary

```typescript
return (
  <div>
    {isLoading ? <Loading /> : <Content />}
    {error && <ErrorMessage error={error} />}
    {selectedClientId ? <Outlet /> : <SelectActiveClient />}
  </div>
);
```

For complex conditions, extract to variables:

```typescript
const renderContent = () => {
  if (isLoading) return <Loading />;
  if (error) return <ErrorMessage error={error} />;
  return <Content />;
};

return <div>{renderContent()}</div>;
```

### Early Returns
Use early returns for guard clauses:

```typescript
export const LayoutRoot = () => {
  const { email, name } = useSelector((state: RootState) => state.auth.user) as User;

  if (!email || !name) return null;

  return (
    <div>
      {/* rest of component */}
    </div>
  );
};
```

### Component Size
- Keep components **under 400 lines**
- Extract subcomponents or logic into separate files when needed

### Display Names
Set `displayName` for components using `forwardRef`:

```typescript
Button.displayName = 'Button';
```

### Consistent Exports
Always use named exports for components:

```typescript
export const MyComponent = () => ...
```

### One Component per File
Each React component declared in its own `.tsx` file.

---

## 5. Hooks

### Custom Hooks
- Use **kebab-case** for hook files: `use-debounce.ts`
- Prefix with `use`: `useDebounce`, `useTokenRefresh`
- Return values, not JSX

```typescript
export function useDebounce<T>(value: T, delay: number = 500): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}
```

### Dependency Arrays
- **Minimize dependencies**: destructure objects to primitives
- **Memoize reference types** if you must include them
- Only disable ESLint with justification comments

```typescript
const { id, name } = user;

useEffect(() => {
  fetchUser(id);
}, [id]);

useEffect(() => {
  processData(data);
}, [JSON.stringify(data)]);
```

### Memoization
Use `useMemo` and `useCallback` only when:
- Values/functions are expensive to compute
- They are dependencies in other hooks
- Performance profiling indicates a need

```typescript
const expensiveValue = useMemo(() => {
  return computeExpensiveValue(data);
}, [data]);

const handleClick = useCallback(() => {
  onClick(id);
}, [id, onClick]);
```

---

## 6. State Management (Redux)

### Slice Structure
Each Redux slice should have:

```
auth/
  ├── types.ts      # Type definitions
  ├── selectors.ts  # Memoized selectors
  ├── actions.ts    # Extended async actions
  └── index.ts      # Slice definition
```

### Slice Definition
```typescript
const slice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    setUser: (state, action: PayloadAction<User | undefined>) => {
      state.user = action.payload;
    },
    setIsAuthenticated: (state, action: PayloadAction<boolean>) => {
      state.isAuthenticated = action.payload;
    },
  },
});

export const { actions } = slice;
export const authReducer = slice.reducer;
```

### Extended Actions
Async operations go in `actions.ts`:

```typescript
export const login = (credentials: LoginCredentials) => {
  return async () => {
    try {
      const { accessToken } = await fetcher<LoginResponse>('/api/auth/login', {
        method: 'POST',
        body: credentials,
      });

      localStorage.setItem('access_token', accessToken);
      const user = await getCurrentUser();

      return { accessToken, user };
    } catch (error) {
      clearTokens();
      setAuthState(undefined, false);
      throw error;
    }
  };
};

export const extendActions = {
  login,
  getCurrentUser,
  checkAuthStatus,
  logout,
};
```

Then merge in `index.ts`:
```typescript
export const authActions = { ...actions, ...extendActions };
```

### Selectors
Use `createSelector` for memoized selectors:

```typescript
export const selectUser = (state: RootState) => state.auth.user;
export const selectIsAuthenticated = (state: RootState) => state.auth.isAuthenticated;
```

### Reducers
- Clearing state action returns the default state, no payload

### Action Creators
Use the standard async template:
- Start loading
- Try/catch API
- Dispatch updates
- End loading

### Usage in Components
```typescript
const user = useSelector((state: RootState) => state.auth.user);
const dispatch = useDispatch();

const handleLogin = () => {
  dispatch(authActions.login(credentials));
};
```

---

## 7. Styling

### Tailwind CSS
- Use Tailwind utility classes for styling
- Use the `cn` utility for conditional classes

```typescript
import { cn } from '@app/utils';

<div className={cn(
  'base-classes',
  isActive && 'active-classes',
  className
)}>
```

### CSS Modules
For complex component-specific styles, use CSS Modules:

```typescript
import styles from './LayoutRoot.module.scss';

<div className={styles.layoutRoot}>
```

### Class Variance Authority (CVA)
Use CVA for component variants:

```typescript
import { cva, type VariantProps } from 'class-variance-authority';

const buttonVariants = cva(BASE_BUTTON_STYLES, {
  variants: {
    variant: {
      default: 'bg-btnGray1 text-btnSecondary',
      primary: 'bg-btnPrimary text-fgWhite',
      secondary: 'bg-btnSecondary text-fgWhite',
    },
    size: {
      default: 'px-4',
      icon: 'w-10 aspect-square',
    },
  },
  defaultVariants: {
    variant: 'default',
    size: 'default',
  },
});

type ButtonProps = VariantProps<typeof buttonVariants> & {
  // ...
};
```

---

## 8. Testing

### Test File Location
- Place test files next to the component: `Button.test.tsx`
- Use Vitest and React Testing Library

### Test Structure
```typescript
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Button } from './Button';

describe('Button component', () => {
  it('renders button with text content', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('handles click events', async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Click me</Button>);

    await user.click(screen.getByText('Click me'));
    expect(handleClick).toHaveBeenCalled();
  });

  it('is disabled when disabled prop is true', () => {
    render(<Button disabled>Disabled Button</Button>);
    expect(screen.getByText('Disabled Button')).toBeDisabled();
  });
});
```

### Test Data Attributes
Use `data-testid` for reliable element selection:

```typescript
<Button data-testid="gear-button" onClick={handleClick}>
  <GearIcon />
</Button>

expect(screen.getByTestId('gear-button')).toBeInTheDocument();
```

---

## 9. Code Organization

### Barrel Exports
Use `index.ts` files for clean imports:

```typescript
// src/components/index.ts
export { Button } from './buttons';
export { Modal } from './modals';
export { Input } from './Input';
export * from './Icons';
```

### Component Folders
For components with subcomponents:

```
ItemComponents/
  ├── ItemComponents.tsx
  └── index.ts
```

### Route Configuration
Centralize routes in a config file:

```typescript
export const WEB_ROUTES = {
  SETUP_PAGE: {
    path: '/setup',
    title: 'Setup',
  },
  LOGIN_PAGE: {
    path: '/login',
    title: 'Login',
  },
};
```

### Constants
Extract magic strings and numbers to constants:

```typescript
const BASE_BUTTON_STYLES = 'h-10 text-base inline-flex items-center...';
const DEBOUNCE_DELAY = 500;
```

---

## 10. Best Practices

### Modern JavaScript
- Use **template literals** for string interpolation
- Use **object shorthand** syntax
- Use **destructuring** for objects and arrays
- Prefer **arrow functions** for callbacks

```typescript
const message = `${user.name} has ${count} items`;

const obj = { value, count };

const { id, name, email } = user;
const [first, second] = items;
```

### Error Handling
Handle errors appropriately:

```typescript
try {
  const data = await fetcher('/api/endpoint');
  return data;
} catch (error) {
  console.error('Failed to fetch:', error);
  throw error;
}
```

### Async/Await
Prefer async/await over Promise chains:

```typescript
const fetchData = async () => {
  const user = await fetcher<User>('/api/me');
  const settings = await fetcher<Settings>('/api/settings');
  return { user, settings };
};
```

### Comments
- Avoid adding comments to code (per project preference)
- Let code be self-documenting through clear naming

### File Extensions
- Use `.tsx` for React components
- Use `.ts` for utilities, types, and non-JSX files
- Use `.test.tsx` for component tests

### ESLint Rules
- Follow ESLint configuration
- Custom rule: `use-props-only` - single Props type should be named `Props`


### Translations
- Non-English keys default to English with `// @todo`

---

## Summary Checklist

When creating a new component:

- [ ] File named in PascalCase (e.g., `Button.tsx`)
- [ ] Component declared as `const ComponentName = ...`
- [ ] Props typed as `type Props = {...}` or `ComponentNameProps`
- [ ] Uses path aliases for imports (`@app/*`)
- [ ] Named exports only
- [ ] Early returns for guard clauses
- [ ] Ternary operators for conditional rendering
- [ ] Default props in function parameters
- [ ] `displayName` set if using `forwardRef`
- [ ] Test file created (`ComponentName.test.tsx`)
- [ ] Exported via `index.ts` if in a folder
- [ ] Under 400 lines (extract if needed)
- [ ] No comments in code
- [ ] TypeScript strict mode compliant
- [ ] ESLint passes
- [ ] Prettier formatted

---

This guide should be followed for all new code. Legacy code should be updated progressively when touched.

