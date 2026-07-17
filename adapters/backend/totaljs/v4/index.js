const total = require('total.js')
total.http('debug')
total.route('/', () => 'Hello Total.js')
total.listen(3000)
