# SPDX-License-Identifier: MPL-2.0
"""Read-only independent input audit; never changes the failed launcher outcome."""
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path.cwd()/'tools/verification'))
from temporal_diagnosis import arms, validate
run=Path(sys.argv[1]); output=Path(sys.argv[2])
if output.resolve().is_relative_to(run.resolve()):
    raise ValueError('Diagnostic output must stay outside the original attempt')
spec=json.loads((run/'cases/experiment.temporal-jitter.protocol/protocol.json').read_text())
result=validate(run/'readbacks/experiment.temporal-jitter.r1',planned_arms=arms(spec),spec=spec,proton=True)
result['scope']='Supplementary input/output oracle only; launcher exit 247 and blocked full jitter acceptance remain unchanged'
with output.open('x') as stream:
    stream.write(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
