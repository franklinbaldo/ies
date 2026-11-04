---
type: ImplementationDecision
title: Ler da árvore já conferida onde o processo está aberto e a natureza dos documentos
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de processo.obter (Refs #59); ies processo NUP passa a publicar aberto_em (do .html do nó PROCESSO) e icone, cancelado e motivo_do_cancelamento de cada DOCUMENTO (a6 a a9), sem requisição a mais e sem falhar o comando por forma desconhecida
selected_strategy: web-situacao-da-arvore-estrita
alternatives: web-situacao-da-arvore-estrita,web-situacao-pelo-texto-livre,web-situacao-pelo-andamento,web-natureza-com-dica,web-natureza-traduzida
evidence_level: capturas-privadas-estrutura-unb-go-pen1-e-testes-sinteticos
live_validation: pass-unb-cancelado-go-pen1-not-run
---

# Onde o processo está aberto e natureza dos documentos

Apêndice da [decisão de `processo.obter`](../processo-obter.md), que
passaria de 20.000 bytes. Contrato:
[situação e natureza](../../../contracts/cli/processo-obter-situacao.md);
estrutura em
[situação na árvore](../../como-o-sei-funciona/situacao-na-arvore.md).

A árvore que `ies processo NUP` já pede e confere traz duas informações
que o comando descartava: o `.html` do nó `PROCESSO` (onde o processo
está aberto e com quem) e os argumentos a6 a a9 dos documentos (ícone e
dica). Lê-las custa **0 requisições**.

## Alternativas consideradas

- **web-situacao-da-arvore-estrita** (escolhida): ler o `.html` da raiz
  pelo leitor de literais e as âncoras `a.ancoraSigla` pelo tokenizador
  de atributos, aceitando só as três formas observadas, peça por peça;
  forma desconhecida vira `aberto_em: null` com motivo.
- **web-situacao-pelo-texto-livre**: publicar o texto do `.html` sem as
  tags. Rejeitada: o agente teria de reparsear uma frase que muda de
  singular para plural, e o aviso de anexação sairia como se fosse
  situação.
- **web-situacao-pelo-andamento**: deduzir as unidades abertas do
  histórico (último "Processo recebido"/"remetido" por unidade).
  Rejeitada: é inferência sobre regras de tramitação, depende do
  histórico completo (+1 POST) e some com `--sem-andamentos`.
- **web-natureza-com-dica**: publicar a dica (a6) quando difere do
  rótulo. Rejeitada na primeira versão (os e-mails trazem endereços, que o
  contrato da árvore recusava publicar); **adotada** em seguida, no mesmo
  PR, por decisão do proprietário (04/11/2026): as instalações de
  treinamento são públicas e os metadados do e-mail têm valor para o
  agente. A dica sai como servida, sem interpretação.
- **web-natureza-traduzida**: traduzir o ícone (`documento_pdf` →
  "externo"). Rejeitada: o significado não foi demonstrado; o radical sai
  opaco e a leitura provável fica marcada como inferência na observação.

## Comparação

| Critério | Estrita | Texto livre | Andamento | Com dica | Traduzida |
| --- | --- | --- | --- | --- | --- |
| Requisições | 0 | 0 | 0 a +2 | 0 | 0 |
| Estrutura publicada | lista de objetos | frase | inferida | sim | sim |
| Metadados do e-mail | não | não | não | **sim** | não |
| Afirma o não observado | não | não | **sim** | não | **sim** |
| Forma nova | `null` + motivo | vaza | — | — | — |

## Decisão

`aberto_em` é lista `{sigla, nome, atribuido_a}` na ordem servida; `[]`
só para "Processo não possui andamentos abertos."; o resto, `null` com
`aberto_em_nao_lido_motivo` e `aberto_em` no começo de `nao_lidos` (a
ordem alfabética já usada). Nunca falha o comando, como as propriedades e
os marcadores não publicados: a árvore já provou a identidade, e uma
frase nova numa versão não observada não deve derrubar a leitura dos
documentos. O processo anexado tem motivo próprio
(`raiz_publica_o_vinculo_de_anexacao`), dado pelo vínculo conferido, não
pela frase.

A leitura é estrita: âncora com exatamente `alt`, `title` e `class`,
`alt` igual a `title`, texto único sem entidade, e nada entre as peças
além de `<br />`, ` (atribuído para …)` e o ponto final. Rigidez escolhida
para não publicar como unidade um link que não seja de unidade.

`icone` sai só com os três ícones iguais na forma `svg/<radical>.svg`.
`cancelado` exige **duas** marcas independentes (ícone
`documento_cancelado` e dica `Documento Cancelado:`). Meia marca
(uma sem a outra, nunca observada) dá `cancelado: null`: afirmar `true`
publicaria um cancelamento sem a segunda prova, e `false` negaria a
marca vista. Ícone fora da forma dá `icone: null` sem recusa. A dica
sai em `dica` quando difere do rótulo (ver web-natureza-com-dica).

**Orçamento.** Nenhuma requisição nova; os testes de orçamento de
`processo_obter` passam sem mudança.

**Efeito.** Nenhum além do GET da árvore já declarado.

## Evidências verificáveis

- Capturas privadas da UnB, de GO e da PEN1 descritas na
  [observação](../../como-o-sei-funciona/situacao-na-arvore.md): seis
  formas do `.html` da raiz e sete radicais de ícone.
- Testes sintéticos: `arvore_processo::tests_situacao`,
  `arvore_processo::tests_natureza` e
  `processo_obter_cli::tests_situacao`.
- [Evidência da operação](../../../contracts/evidencias/processo-obter.md).

## Limitações e revisão

Validação live **PASS** na UnB (unidade única, plural com atribuição,
anexado, quente), sem requisição a mais; documento cancelado, GO e PEN1
**NOT_RUN** ao vivo ([relatório](../../validacao-live/2026-11-04-processo-obter-situacao.md)).
Não observados: processo sobrestado ou concluído em todas as unidades,
5.0.x fora de GO, documento cancelado fora de GO, ícone fora de `svg/`.
Revisar se uma instalação publicar outra frase, âncora sem
`ancoraSigla` ou ícones diferentes entre a7 e a9.
