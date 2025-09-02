/**
 * # Todo Application Frontend
 * 
 * Main React component providing a complete todo management interface with user authentication.
 * Features user registration/login, todo CRUD operations, and persistent authentication state.
 * 
 * ## Features
 * - User authentication (register/login) with JWT tokens
 * - Todo item management (create, read, update, delete)
 * - Persistent login state using localStorage
 * - Real-time UI feedback for all operations
 * - Responsive design with CSS styling
 * 
 * ## Architecture
 * - Single-page application using React hooks
 * - Axios for HTTP API communication
 * - JWT token-based authentication
 * - Local state management for todos and user data
 */

import React, { useState, useEffect } from 'react'
import axios from 'axios'

/**
 * Todo item data structure
 * Represents a single todo item as returned by the API
 */
interface Todo {
  /** Unique identifier for the todo item */
  id: string
  /** Title/description of the todo */
  title: string
  /** Whether the todo has been completed */
  completed: boolean
  /** ISO 8601 formatted creation timestamp */
  created_at: string
}

/**
 * User authentication data structure
 * Contains user information and authentication token
 */
interface User {
  /** Username for display purposes */
  username: string
  /** JWT token for API authentication */
  token: string
}

/** Base URL for all API requests (proxied by nginx) */
const API_BASE = '/api'

/**
 * Main application component
 * 
 * Manages the entire application state including authentication and todo management.
 * Renders different views based on authentication status.
 * 
 * @returns JSX element containing the complete application interface
 */
function App() {
  // Authentication state
  const [user, setUser] = useState<User | null>(null)
  const [todos, setTodos] = useState<Todo[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')

  // Auth form state
  const [authMode, setAuthMode] = useState<'login' | 'register'>('login')
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')

  // Todo form state
  const [newTodo, setNewTodo] = useState('')

  /**
   * Initialize application state on component mount
   * Checks for saved authentication data and loads user's todos
   */
  useEffect(() => {
    const savedUser = localStorage.getItem('user')
    if (savedUser) {
      const userData = JSON.parse(savedUser)
      setUser(userData)
      fetchTodos(userData.token)
    }
  }, [])

  /**
   * Display temporary success or error messages
   * 
   * @param msg - Message to display
   * @param type - Message type ('error' or 'success')
   */
  const showMessage = (msg: string, type: 'error' | 'success') => {
    if (type === 'error') {
      setError(msg)
      setSuccess('')
    } else {
      setSuccess(msg)
      setError('')
    }
    setTimeout(() => {
      setError('')
      setSuccess('')
    }, 3000)
  }

  /**
   * Handle user authentication (login or registration)
   * 
   * Validates input, sends authentication request to API, and handles response.
   * On successful login, saves user data and fetches todos.
   * 
   * @param e - Form submission event
   */
  const handleAuth = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!username.trim() || password.length < 6) {
      showMessage('Username required and password must be at least 6 characters', 'error')
      return
    }

    setLoading(true)
    try {
      if (authMode === 'register') {
        await axios.post(`${API_BASE}/register`, { username, password })
        showMessage('Registration successful! Please login.', 'success')
        setAuthMode('login')
      } else {
        const response = await axios.post(`${API_BASE}/login`, { username, password })
        const userData = { username, token: response.data.token }
        setUser(userData)
        localStorage.setItem('user', JSON.stringify(userData))
        showMessage('Login successful!', 'success')
        fetchTodos(userData.token)
      }
      setUsername('')
      setPassword('')
    } catch (err: any) {
      showMessage(err.response?.data || 'Authentication failed', 'error')
    }
    setLoading(false)
  }

  /**
   * Handle user logout
   * 
   * Clears user state, todos, and removes authentication data from localStorage.
   */
  const handleLogout = () => {
    setUser(null)
    setTodos([])
    localStorage.removeItem('user')
    showMessage('Logged out successfully', 'success')
  }

  /**
   * Fetch todos from the API
   * 
   * Retrieves all todos for the authenticated user and updates local state.
   * 
   * @param token - JWT authentication token
   */
  const fetchTodos = async (token: string) => {
    try {
      const response = await axios.get(`${API_BASE}/todos`, {
        headers: { Authorization: `Bearer ${token}` }
      })
      setTodos(response.data)
    } catch (err: any) {
      showMessage(err.response?.data || 'Failed to fetch todos', 'error')
    }
  }

  /**
   * Handle adding a new todo item
   * 
   * Validates input, sends create request to API, and refreshes todo list.
   * 
   * @param e - Form submission event
   */
  const handleAddTodo = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newTodo.trim()) return

    setLoading(true)
    try {
      await axios.post(`${API_BASE}/todos`, 
        { title: newTodo },
        { headers: { Authorization: `Bearer ${user!.token}` } }
      )
      setNewTodo('')
      fetchTodos(user!.token)
      showMessage('Todo added!', 'success')
    } catch (err: any) {
      showMessage(err.response?.data || 'Failed to add todo', 'error')
    }
    setLoading(false)
  }

  /**
   * Handle toggling todo completion status
   * 
   * Sends update request to API to toggle the completed status of a todo item.
   * 
   * @param id - Todo item ID
   * @param completed - Current completion status
   */
  const handleToggleTodo = async (id: string, completed: boolean) => {
    try {
      await axios.put(`${API_BASE}/todos/${id}`,
        { completed: !completed },
        { headers: { Authorization: `Bearer ${user!.token}` } }
      )
      fetchTodos(user!.token)
    } catch (err: any) {
      showMessage(err.response?.data || 'Failed to update todo', 'error')
    }
  }

  /**
   * Handle deleting a todo item
   * 
   * Sends delete request to API and refreshes todo list.
   * 
   * @param id - Todo item ID to delete
   */
  const handleDeleteTodo = async (id: string) => {
    try {
      await axios.delete(`${API_BASE}/todos/${id}`, {
        headers: { Authorization: `Bearer ${user!.token}` }
      })
      fetchTodos(user!.token)
      showMessage('Todo deleted!', 'success')
    } catch (err: any) {
      showMessage(err.response?.data || 'Failed to delete todo', 'error')
    }
  }

  // Render authentication form if user is not logged in
  if (!user) {
    return (
      <div className="container">
        <div className="header">
          <h1>Todo App</h1>
          <p>Please {authMode} to continue</p>
        </div>

        <form className="auth-form" onSubmit={handleAuth}>
          <h2>{authMode === 'login' ? 'Login' : 'Register'}</h2>
          
          <div className="form-group">
            <label>Username:</label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
            />
          </div>

          <div className="form-group">
            <label>Password:</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              minLength={6}
            />
          </div>

          <button type="submit" className="btn" disabled={loading}>
            {loading ? 'Loading...' : authMode === 'login' ? 'Login' : 'Register'}
          </button>

          <button
            type="button"
            className="btn"
            onClick={() => setAuthMode(authMode === 'login' ? 'register' : 'login')}
          >
            Switch to {authMode === 'login' ? 'Register' : 'Login'}
          </button>
        </form>

        {error && <div className="error">{error}</div>}
        {success && <div className="success">{success}</div>}
      </div>
    )
  }

  // Render main todo application interface for authenticated users
  return (
    <div className="container">
      <div className="header">
        <h1>Todo App</h1>
      </div>

      <div className="user-info">
        <span>Welcome, {user.username}!</span>
        <button className="btn btn-danger" onClick={handleLogout}>
          Logout
        </button>
      </div>

      <form className="add-todo" onSubmit={handleAddTodo}>
        <input
          type="text"
          placeholder="Add a new todo..."
          value={newTodo}
          onChange={(e) => setNewTodo(e.target.value)}
        />
        <button type="submit" className="btn" disabled={loading}>
          {loading ? 'Adding...' : 'Add Todo'}
        </button>
      </form>

      {error && <div className="error">{error}</div>}
      {success && <div className="success">{success}</div>}

      <div className="todo-list">
        {todos.length === 0 ? (
          <div className="loading">No todos yet. Add one above!</div>
        ) : (
          todos.map((todo) => (
            <div key={todo.id} className={`todo-item ${todo.completed ? 'completed' : ''}`}>
              <input
                type="checkbox"
                checked={todo.completed}
                onChange={() => handleToggleTodo(todo.id, todo.completed)}
              />
              <span className="todo-text">{todo.title}</span>
              <div className="todo-actions">
                <button
                  className="btn btn-danger"
                  onClick={() => handleDeleteTodo(todo.id)}
                >
                  Delete
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  )
}

export default App