import React from 'react'

export default function Button({ children, variant = 'primary', onClick, type = 'button', disabled = false }) {
  const baseStyles = {
    padding: '0.6em 1.2em',
    borderRadius: '8px',
    border: '1px solid transparent',
    fontSize: '1em',
    fontWeight: 500,
    cursor: disabled ? 'not-allowed' : 'pointer',
    opacity: disabled ? 0.6 : 1,
    transition: 'border-color 0.25s, background-color 0.25s',
  }

  const variants = {
    primary: { backgroundColor: '#646cff', color: '#fff' },
    secondary: { backgroundColor: '#213547', color: '#fff' },
    outline: { backgroundColor: 'transparent', border: '1px solid #646cff', color: '#646cff' },
    ghost: { backgroundColor: 'transparent', color: '#213547' },
  }

  const style = { ...baseStyles, ...variants[variant] || variants.primary }

  return (
    <button type={type} onClick={onClick} disabled={disabled} style={style}>
      {children}
    </button>
  )
}
