---
type: ImplementationDecision
title: Sobrestar e retomar por ação Web assinada com situação verificada
operation: processo.sobrestar
implementation: partial
status: adopted-for-scope
scope: dois comandos Web, escrita única e verificação offline; ensaio autenticado pendente
selected_strategy: barra-assinada-com-situacao-lida
alternatives: barra-assinada-com-situacao-lida,rest-nao-observado,heuristica-textual
evidence_level: cassettes-todos-sete-instalacoes-e-testes-sinteticos
live_validation: not-run
---

# Decisão — sobrestar e retomar (#269)

## Contexto

A barra da árvore do processo publica `procedimento_sobrestar` como
URL assinada. A retomada é `removerSobrestamentoProcesso();` e o
endereço de escrita está na tela do nó. A unidade ativa é observada
na tela do processo (`lnkInfraUnidade`); o estado está no HTML
do nó raiz (`AbertoEm::Unidades` ou `AbertoEm::Sobrestado`).
A leitura não pode supor a instalação nem a unidade a partir de cache.

## Alternativas consideradas

1. **Barra assinada e situação lida** (escolhida): reutiliza os
   leitores de HTML e URL assinada já tipados, confere ID e unidade,
   nenhuma requisição extra na prévia.
2. **REST não observado**: não há rota demonstrada com equivalência
   de autenticação, estado e efeitos.
3. **Heurística textual**: decidir por ícone ou texto da barra
   permite falsos positivos e operação em outro processo.

## Comparação

| Critério | Barra assinada | REST | Heurística |
| --- | --- | --- | --- |
| Evidência nas sete instalações do Todos | sim | não | parcial |
| Identidade por ID assinado | sim | a provar | não |
| URL construida pela CLI | não | a provar | possivelmente |
| Prévia a frio | 3 GET | desconhecido | desconhecido |

## Decisão

Usar `arvore_processo::barra_do_processo` e
`arvore_processo::endereco` para extrair a ação do próprio nó
`PROCESSO`; reutilizar `abrir_para_escrever` sem o link da árvore
guardado para conferir a unidade. Não executar JavaScript, não usar
regex, não seguir 302, não construir links com `infra_hash`.

Entregues em #279: CLI, motivo obrigatório, situação, barra,
prévia funcional sem POST e testes de identidade e contagem.
A #284 implementa confirmação: sobrestar pela tela do SEI com
formulário derivado e POST único; retomar pelo GET de função
publicada na tela do nó. Ambos têm releitura obrigatória da
situação e recusam escrever fora da unidade. O POST consome
`SubmissaoDerivada`, sem retry, sem executar JS e sem seguir 302.
Transporte roteirizado sintético: seis pedidos a frio, sendo
um único envio, inclusive na falha e na reconciliação.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-sobrestar.md):
  condições de entrada, estados recusados e subcódigos.
- [Matriz da operação](../../contracts/evidencias/processo-sobrestar.md):
  sete cassettes do Todos, não é execução do novo código.
- [Teste Rust da CLI](../../tests/cli_processo_sobrestar.rs):
  uso inválido e falta de sessão para confirmar.
- [Testes do leitor](../../src/arvore_processo/tests_sobrestar.rs)
  e [testes de fluxo](../../src/processo_sobrestar/tests.rs):
  URL assinada e três GET exatos a frio. [Testes de escrita](../../src/processo_sobrestar/tests_escrita.rs)
  exercitam orçamento de 6 requisições, um POST, sucesso, recusa e escrita não confirmada.
- [Live](../validacao-live/2026-11-05-processo-sobrestar.md):
  NOT_RUN, zero requisições e nenhum resíduo.
- [Issue #269](https://github.com/franklinbaldo/ies/issues/269),
  [PR integrada #279](https://github.com/franklinbaldo/ies/pull/279)
  e [PR Draft #284](https://github.com/franklinbaldo/ies/pull/284).

## Limitações e revisão

O caminho de escrita está implementado em testes sintéticos, mas
**replay do acervo e ensaio autenticado em treinamento permanecem NOT_RUN**.
Nunca usar esta evidência offline para afirmar sucesso numa instalação.
Para validar ao vivo: prévia, um único `--confirmar` e leitura
independente; relatar requisições, resultados e eventual resíduo.
Não declarar a issue concluída enquanto a validação faltar.
