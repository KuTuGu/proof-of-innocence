const { prove } = require("../pkg/circuit");
const { Command } = require('commander');
const { writeFile } = require('fs/promises');

const program = new Command();

program
  .name('proof-of-innocence')
  .description('Proof of Innocence built on Tornado Cash')
  .version('0.0.1');

program.command('prove')
  .description('Generate zk prove for zkVM')
  .requiredOption('-n, --noteList  Array<string...>', 'tornado note list, required')
  .requiredOption('-b, --blockList Array<string...>', 'block commitment list, required')
  .option('-vm, --zkVM <string>', 'which zkVM to use, default risc0', 'risc0')
  .action(params => {
    prove(params.noteList, params.blockList)
      .then(data => {
        writeFile(`${__dirname}/../output/proof.json`, data);
      })
      .catch(err => {
        console.error("\x1B[31m%s\x1B[0m", `\nError: ${err}`);
      });
  });

program.parse();
