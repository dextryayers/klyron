import React from 'react'

export default function Card({ title, children, footer, image, className, variant = 'elevated' }) {
  const variantClasses = {
    elevated: 'shadow-lg',
    outlined: 'border border-gray-200',
    flat: 'bg-gray-50',
  }

  return (
    <div className={`rounded-xl bg-white overflow-hidden ${variantClasses[variant] || variantClasses.elevated} ${className || ''}`}>
      {image && <img src={image} alt="" className="w-full h-48 object-cover" />}
      <div className="p-6">
        {title && <h3 className="text-lg font-semibold mb-3">{title}</h3>}
        <div className="text-gray-600">{children}</div>
      </div>
      {footer && (
        <div className="px-6 py-4 border-t border-gray-100 bg-gray-50 flex items-center gap-2">
          {footer}
        </div>
      )}
    </div>
  )
}
