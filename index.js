const nativeBinding = require('./dist/native')

module.exports = nativeBinding
module.exports.getWindowsInfo = nativeBinding.getWindowsInfo
module.exports.sendNotification = nativeBinding.sendNotification
