import { useState } from 'react'
import reactLogo from '../assets/react.svg'
import '../App.css'

function Home() {
  const [count, setCount] = useState(0)

  return (
    <div className="home">
      <header className="home-header">
        <img src={reactLogo} className="logo react" alt="React logo" />
        <h1>{{ name }}</h1>
        <p>{{ description }}</p>
        <div className="card">
          <button onClick={() => setCount((c) => c + 1)}>
            count is {count}
          </button>
        </div>
      </header>
    </div>
  )
}

export default Home
