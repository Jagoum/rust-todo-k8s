import React, { useState, useEffect } from 'react'
import axios from 'axios'

interface Todo {
  id: string
  title: string
  completed: boolean
  created_at: string
}

interface User {
  username: string
  token: string
}

const API_BASE = '/api'

function App() {
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

  useEffect(() => {
    const savedUser = localStorage.getItem('user')
    if (savedUser) {
      const userData = JSON.parse(savedUser)
      setUser(userData)
      fetchTodos(userData.token)
    }
  }, [])

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

  const handleLogout = () => {
    setUser(null)
    setTodos([])
    localStorage.removeItem('user')
    showMessage('Logged out successfully', 'success')
  }

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