const getDocumentTransitionsFixture = require('../../../../../../../lib/test/fixtures/getDocumentTransitionsFixture');
const { default: loadWasmDpp } = require('../../../../../../../dist');

let findDuplicatesById;

describe('findDuplicatesById', () => {
  let rawDocumentTransitions;

  beforeEach(async () => {
    ({
      findDuplicatesById,
    } = await loadWasmDpp());
    rawDocumentTransitions = getDocumentTransitionsFixture().map((t) => t.toObject());
  });

  it('should return empty array if there are no duplicated Documents', () => {
    const result = findDuplicatesById(rawDocumentTransitions);

    expect(result).to.be.an('array');
    expect(result).to.have.lengthOf(0);
  });

  it('should return duplicated Documents', () => {
    rawDocumentTransitions.push(rawDocumentTransitions[0]);

    const result = findDuplicatesById(rawDocumentTransitions);

    expect(result).to.be.an('array');
    expect(result).to.have.lengthOf(2);
    expect(result).to.have.deep.members([
      rawDocumentTransitions[0],
      rawDocumentTransitions[0],
    ]);
  });
});
