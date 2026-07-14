import React from 'react'

export default function Card({ title, children, footer, image, variant = 'elevated' }) {
  const cardStyle = {
    borderRadius: '12px',
    overflow: 'hidden',
    backgroundColor: '#fff',
    ...(variant === 'elevated'
      ? { boxShadow: '0 4px 12px rgba(0,0,0,0.1)' }
      : variant === 'outlined'
        ? { border: '1px solid #e0e0e0' }
        : {}),
  }

  return (
    <div style={cardStyle}>
      {image && <img src={image} alt="" style={{ width: '100%', height: '200px', objectFit: 'cover' }} />}
      <div style={{ padding: '1.5rem' }}>
        {title && <h3 style={{ margin: '0 0 0.75rem 0', fontSize: '1.25rem' }}>{title}</h3>}
        <div>{children}</div>
      </div>
      {footer && (
        <div style={{ padding: '1rem 1.5rem', borderTop: '1px solid #e0e0e0', backgroundColor: '#f9f9f9' }}>
          {footer}
        </div>
      )}
    </div>
  )
}
