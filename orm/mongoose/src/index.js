import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createORMAdapter() {
  return {
    name: 'mongoose',
    detect(dir) {
      try {
        const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'))
        return pkg?.dependencies?.mongoose || pkg?.devDependencies?.mongoose
      } catch { return false }
    },
    supportedVersions: ['8.0', '8.9'],
    defaultVersion: '8.9',
    kind: 'ORM',

    scaffold(name, options) {
      return scaffoldMongoose(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldMongoose(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'models'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'node src/index.js', start: 'node src/index.js' },
    dependencies: { mongoose: '^8.9.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'index.js'), `import mongoose from 'mongoose'
import User from './models/User.js'
import Post from './models/Post.js'

await mongoose.connect(process.env.MONGODB_URI || 'mongodb://localhost:27017/${name}')

const user = await User.create({ email: 'test@example.com', name: 'Test User' })
console.log('Created user:', user)

const post = await Post.create({ title: 'Hello World', content: 'My first post', author: user._id })
console.log('Created post:', post)

await mongoose.disconnect()
`)

  writeFileSync(join(projectDir, 'src', 'models', 'User.js'), `import mongoose from 'mongoose'

const userSchema = new mongoose.Schema({
  email: { type: String, required: true, unique: true },
  name: { type: String },
  createdAt: { type: Date, default: Date.now },
  updatedAt: { type: Date, default: Date.now }
}, { timestamps: true })

export default mongoose.model('User', userSchema)
`)

  writeFileSync(join(projectDir, 'src', 'models', 'Post.js'), `import mongoose from 'mongoose'

const postSchema = new mongoose.Schema({
  title: { type: String, required: true },
  content: { type: String },
  published: { type: Boolean, default: false },
  author: { type: mongoose.Schema.Types.ObjectId, ref: 'User', required: true },
  createdAt: { type: Date, default: Date.now },
  updatedAt: { type: Date, default: Date.now }
}, { timestamps: true })

export default mongoose.model('Post', postSchema)
`)

  writeFileSync(join(projectDir, '.env'), 'MONGODB_URI="mongodb://localhost:27017/myapp"\n')
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nMongoose ODM\n\nnpm run dev\n`)
}
