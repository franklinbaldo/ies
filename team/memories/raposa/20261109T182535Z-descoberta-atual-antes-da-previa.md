---
type: AgentMemory
title: "Descoberta atual da unidade antes da prévia de atribuição"
description: "PR #595: ferramenta manual lê recebidos e até 3 processos por CLI para buscar ausência observada de atribuição na unidade ativa; RED real, inspeção de cassettes PASS e CI final em revalidação."
agent: raposa
timestamp: "2026-11-09T18:25:35Z"
tags: [ux, processo-atribuir, treinamento, descoberta, cassette-first, red-green]
---

# Para retomar

**Fato:** os dois NUPs históricos da UnB falharam na prévia; não reutilizar sua elegibilidade antiga. A [PR #595](https://github.com/franklinbaldo/ies/pull/595) implementa descoberta **somente por consultas** da CLI: primeira página de `unidade recebidos` e até três `processo NUP`, escolhendo apenas identidade/unidade coincidentes e ausência de atribuição explicitamente observada. O resultado é candidato para **nova prévia**, nunca autorização automática de POST. Sem alteração da CLI pública ou do anti-IES.

**Provas:** [RED #42672580826](https://github.com/franklinbaldo/ies/actions/runs/42672580826) mostrou 7 erros por função ausente; [inspeção de dois cards UnB #42672793875](https://github.com/franklinbaldo/ies/actions/runs/42672793875) conferiu hashes de 39 interações, sem replay ou paridade causal. [Primeiro CI pós-GREEN #42673237270](https://github.com/franklinbaldo/ies/actions/runs/42673237270) ficou marcado como falha global apesar de três jobs retornados como verdes; novo [CI #42673770524](https://github.com/franklinbaldo/ies/actions/runs/42673770524) foi disparado para diagnosticar. Conferir o resultado antes de solicitar integração.

**Limite:** não houve validação externa: criação do workflow temporário foi bloqueada pela ferramenta. Não alegar prova live nem repetir consultas falhas. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6747289891), a distinção entre constar no catálogo e estar acessível agora é uma analogia útil, não uma observação adicional do SEI.
