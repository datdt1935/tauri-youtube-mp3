# 🚀 Unified Coding Standards & Best Practices

---

## 1. **File and Folder Naming**

✅ **Components**

- Use **Pascal case** for React component files and folders (e.g., `CardDetail.tsx`, `UserProfile/`).
- Each component goes in **its own file**.
- If a component has subcomponents, place them in a folder named after the parent component and re-export in `index.tsx`.

✅ **Helpers, Hooks, and Utilities**

- Use **kebab-case** (e.g., `filter-context.ts`, `use-auth.ts`).

✅ **Redux**

- Use `camelCase` for feature folders (e.g., `user`).
- File naming inside a slice:
  - `types.ts`
  - `selectors.ts`
  - `actions.ts`
  - `index.ts`

✅ **Branch Naming**

- `TASKID-description`: `M3D-100-addAuthFlow`

---

## 2. **React Components**

✅ **Props Typing**

- **Local use**: `type Props = {...}`
- **Shared use**: `ComponentNameProps`

✅ **Component Declaration**

- Always declare components as `const ComponentName = ...`.

✅ **JSX Return**

- Return JSX **directly**.
- Use **ternary operators** for conditional rendering, **never `if`** inside return.
- For mutually exclusive branches, prefer single ternary:
  ```tsx
  {
    isMobile ? <Hamburger /> : <Navbar />;
  }
  ```
- For complex nested JSX, extract into separate components.

✅ **Default Props**

- Always assign defaults:
  ```tsx
  const MyComponent = ({ name = "", count = 0 }: Props) => ...
  ```

✅ **Memoization**

- Use `useMemo` and `useCallback` **only if**:
  - Values/functions are expensive or
  - They are dependencies in other hooks.

✅ **Dependency Arrays**

- **Minimize dependencies**: destructure objects to primitives.
- **Memoize reference types** if you must include them.
- **Disable ESLint** only with justification comments.

---

## 3. **TypeScript Types**

✅ **Separation**

- Shared types live in `types/` folder.
- Prefer `type` unless you **must** `implements`.

✅ **Boolean Naming**

- Always prefix with `is` or `has`:
  - `isLoaded`
  - `hasPermission`

✅ **Prefix**
Avoid Using the 'I' Prefix for Interfaces

---

### When to Use `type` Instead of `interface`

- Use `type` for:
  - Unions and intersections (e.g., `type Status = "open" | "closed"`)
  - Composing multiple types together (e.g., `type UserWithRole = User & Role`)
  - Function signatures (e.g., `type ClickHandler = (e: MouseEvent) => void`)
  - Aliasing primitives, tuples, or other types

- Use `interface` for:
  - Defining the shape of objects, especially for public APIs or when you expect extension via declaration merging

**Benefits of Using `type`:**

- More flexible: can represent primitives, unions, intersections, and tuples, not just object shapes
- Easily compose multiple types using `&` and `|`
- More concise and readable for simple object shapes or unions
- Prevents accidental merging, making types more predictable

**Example:**

```ts
// Prefer type for unions and intersections
type Status = 'open' | 'closed';
type UserWithRole = User & Role;

// Prefer type for function signatures
type ClickHandler = (e: MouseEvent) => void;

// Use interface for extensible object shapes
interface User {
  id: string;
  name: string;
}
```

---

## 4. **Modern JavaScript Practices**

✅ ** Object Destructuring**

BAD

```
import React from 'react';
...
React.forwardRef(() => {})
```

GOOD:

```
import { forwardRef } from 'react';
...
forwardRef(() => {})

```

✅ **Destructuring**

- Always destructure imports and props:
  ```tsx
  import { useState } from 'react';
  const { id, name } = props;
  ```

✅ **Simplified Returns**

- For single-line arrow functions:
  ```ts
  const total = useMemo(() => calculateTotal(), [items]);
  ```

✅ **Simplified Object Syntax**

- Use shorthand:
  ```ts
  const obj = { value, count };
  ```

✅ **Rupture Clause First**

- Early returns:
  ```ts
  if (!isValid) return;
  ```

✅ **Template Literals**

- Always use backticks:
  ```ts
  `${value} units`;
  ```

---

## 5. **Redux**

✅ **Structure**

- Each slice has:
  - `types.ts`
  - `selectors.ts`
  - `actions.ts`
  - `index.ts`

✅ **Reducers**

- Clearing state action returns the default state, no payload.

✅ **Selectors**

- Memoize with `createSelector`.

✅ **Action Creators**

- Use the standard async template:
  - Start loading
  - Try/catch API
  - Dispatch updates
  - End loading

---

## 6. **Team Workflow**

✅ **Rule Application**

- New code must follow all rules.
- Legacy code updated progressively when touched.

✅ **Pull Requests**

- Minimum **3 approvals**.
- Reviewers **resolve** their own threads after fixes.

✅ **Translations**

- Non-English keys default to English with `// @todo`.

---

## 7. **Miscellaneous**

✅ **One Component per File**

- Each React component declared in its own `.tsx`.

✅ **Consistent Exports**

- Always use named exports for components:
  ```ts
  export const MyComponent = () => ...
  ```

✅ **File Size**

- Keep components **<400 lines**.

✅ **Barrel Files**

- Use `index.ts` for clean imports in folders.
