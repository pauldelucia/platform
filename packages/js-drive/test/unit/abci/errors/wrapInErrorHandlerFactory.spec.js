const wrapInErrorHandlerFactory = require('../../../../lib/abci/errors/wrapInErrorHandlerFactory');
const LoggerMock = require('../../../../lib/test/mock/LoggerMock');
const InternalAbciError = require('../../../../lib/abci/errors/InternalAbciError');
const InvalidArgumentAbciError = require('../../../../lib/abci/errors/InvalidArgumentAbciError');
const VerboseInternalAbciError = require('../../../../lib/abci/errors/VerboseInternalAbciError');
const DPPValidationAbciError = require('../../../../lib/abci/errors/DPPValidationAbciError');

describe('wrapInErrorHandlerFactory', () => {
  let loggerMock;
  let methodMock;
  let request;
  let handler;
  let wrapInErrorHandler;
  let MissingStateTransitionTypeError;

  before(function before() {
      MissingStateTransitionTypeError = this.dppWasm.MissingStateTransitionTypeError;
  });

  beforeEach(function beforeEach() {
    request = {
      tx: Buffer.alloc(0),
    };

    loggerMock = new LoggerMock(this.sinon);

    wrapInErrorHandler = wrapInErrorHandlerFactory(loggerMock, true);
    methodMock = this.sinon.stub();

    handler = wrapInErrorHandler(
      methodMock,
    );
  });

  it('should respond with internal error code if any Error is thrown in handler and respondWithInternalError enabled', async () => {
    handler = wrapInErrorHandler(
      methodMock, { respondWithInternalError: true },
    );

    const error = new Error('Custom error');

    methodMock.throws(error);

    const response = await handler(request);

    const expectedError = new InternalAbciError(error);

    expect(response).to.deep.equal(expectedError.getAbciResponse());
  });

  it('should respond with internal error code if an InternalAbciError is thrown in handler and respondWithInternalError enabled', async () => {
    handler = wrapInErrorHandler(
      methodMock, { respondWithInternalError: true },
    );

    const data = { sample: 'data' };
    const error = new InternalAbciError(new Error(), data);

    methodMock.throws(error);

    const response = await handler(request);

    expect(response).to.deep.equal(error.getAbciResponse());
  });

  it('should respond with invalid argument error if it is thrown in handler', async () => {
    const data = { sample: 'data' };
    const error = new InvalidArgumentAbciError('test', data);

    methodMock.throws(error);

    const response = await handler(request);

    expect(response).to.deep.equal(error.getAbciResponse());
  });

  it('should respond with verbose error containing message and stack in debug mode', async () => {
    wrapInErrorHandler = wrapInErrorHandlerFactory(loggerMock, false);

    const error = new Error('Custom error');

    methodMock.throws(error);

    handler = wrapInErrorHandler(
      methodMock, { respondWithInternalError: true },
    );

    const response = await handler(request);

    const expectedError = new VerboseInternalAbciError(
      new InternalAbciError(error),
    );

    expect(response).to.deep.equal(expectedError.getAbciResponse());
  });

  it('should respond with error if method throws DPPValidationAbciError', async () => {
    const dppValidationError = new DPPValidationAbciError(
      'Some error',
      new MissingStateTransitionTypeError(),
    );

    methodMock.throws(dppValidationError);

    handler = wrapInErrorHandler(
      methodMock, { respondWithInternalError: true },
    );

    const response = await handler(request);

    expect(response).to.deep.equal(dppValidationError.getAbciResponse());
  });
});
