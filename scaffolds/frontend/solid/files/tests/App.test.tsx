import { describe, it, expect } from 'vitest'
import { render, screen } from '@solidjs/testing-library'
import App from '../src/App'

describe('App', () => {
  it('renders heading', () => {
    render(() => <App />)
    expect(screen.getByRole('heading')).toHaveTextContent('{{ name }}')
  })
})
