---
type: Guide
title: Validação da criação de documento interno (issue 246)
description: Resultado das verificações e dos passos ainda não executados
---

# Documento criar — 05/11/2026

Operação: `documento.criar`, issue
[246](https://github.com/franklinbaldo/ies/issues/246).

| Ensaio | Resultado | Requisições |
| --- | --- | --- |
| Leitura das fontes do Todos e das capturas | PASS documental | 0 |
| Testes sintéticos do núcleo Rust | Dependente do CI da PR #248 | 0 |
| Replay integral das cassettes públicas | NOT_RUN | 0 |
| Prévia Web em treinamento | NOT_RUN | 0 |
| Criação ao vivo no SEI | NOT_RUN — sem autorização na issue | 0 |

## Alcance

Os testes sintéticos do núcleo conferem seleção de tipo e diferença
entre árvores. O comando da CLI ainda não está integrado nem disponível.
O resultado do CI não deve ser confundido com prévia ao vivo, replay
real do acervo ou sucesso de escrita. Nenhuma requisição foi enviada
ao SEI nesta frente.

## Para retomar

Implementar o fluxo Web completo com os links servidos, cadastro e
formulário avaliados, e depois testar em treinamento obedecendo ao
orçamento e intervalos da issue. A escrita em treinamento exige a
frase literal **escrita autorizada** escrita pelo proprietário na
issue #246; sem isso, permanece NOT_RUN.
