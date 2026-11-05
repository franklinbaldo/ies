---
type: OperationEvidence
title: processo.criar
operation: processo.criar
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: false
upstream_steps: 235
unresolved_steps: 0
---

# processo.criar

Origem: matriz de demonstrações do Todos. IES Rust: fluxo Web parcial com testes de transporte sintético, sem paridade live/replay comprovada.
Os estados abaixo pertencem ao Todos, não são vereditos do IES.

As referências #N são passos de roteiro; o link leva à revisão com
o mesmo cenário, mas ainda não certifica igualdade de revisão.
Confronte os hashes com o manifesto atual antes de alegar paridade.

## Evidência por instalação

- [unb](detalhes/processo-criar/unb.md) — demonstrada
- [pen1](detalhes/processo-criar/pen1.md) — demonstrada
- [pen2](detalhes/processo-criar/pen2.md) — demonstrada
- [ufrj](detalhes/processo-criar/ufrj.md) — demonstrada
- [pen3](detalhes/processo-criar/pen3.md) — demonstrada
- [pen4](detalhes/processo-criar/pen4.md) — demonstrada
- [go](detalhes/processo-criar/go.md) — demonstrada

## Limite probatório

Consulte os links por instalação, a cassette original e a releitura pós-efeito.


## Avanço sintético da PR #283

O IES agora reconhece a hipótese legal publicada pelo AJAX do cadastro
restrito, sem executar scripts; recusa seleções não únicas antes da
escrita. Cenários: `src/processo_criar/tests_hipotese.rs` e
`src/processo_criar/cadastro.rs`. Prévia sintética 4.0 com AJAX:
**4 pedidos**; confirmação sintética: **7 pedidos**, um único
POST criador e duas leituras posteriores. Os exemplos vêm da
captura `todos@323eb01`, não de replay da cassette original.

`ies_implemented: false` permanece até demonstração de todos os
critérios da issue #245, incluindo replay e validação ao vivo.
