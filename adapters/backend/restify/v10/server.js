const restify = require('restify')
const server = restify.createServer()
server.get('/', (req, res, next) => { res.send('Hello Restify'); next() })
server.listen(3000)
