const nativeBinding = require('./dist/native')

module.exports = nativeBinding
module.exports.add = nativeBinding.add
module.exports.sendNotification = nativeBinding.sendNotification
