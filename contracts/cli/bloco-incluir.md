---
type: CommandContract
title: Incluir documento ou processo em bloco pela interface Web
description: Contrato público unificado de bloco.incluir, com duas transições distintas e confirmação verificável
operation: bloco.incluir
command: ies bloco incluir BLOCO --processo NUP|--documento DOC [--confirmar]
effect: write
implementation: stub
---

# `ies bloco incluir BLOCO --processo NUP | --documento DOC [--confirmar]`

**Contrato aprovado de gramática:** decisão do proprietário registrada em
[#381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6746036266),
consolidando a inclusão de documento em bloco de assinatura (#381) e de
processo em bloco interno/de reunião (#343). **O comando ainda não está
implementado:** este contrato não torna nenhuma confirmação executável.
Conservar a CLI exclusivamente Web, sem REST/SOAP, nem URL/hashes adivinhados.

## Entrada e resolução de identidade

- `BLOCO`: número decimal positivo publicado pela lista da unidade.
- Escolher **exatamente uma** opção: `--documento DOC` (número SEI opaco)
  ou `--processo NUP` (identidade processual opaca). Ausência das duas,
  ambas ou número de bloco inválido: erro de uso (saída 2), antes da sessão
  ou rede. `--json` e `--unidade` continuam globais.
- **Documento:** `BLOCO` precisa designar inequivocamente um bloco de
  **assinatura** e `DOC` precisa resolver-se a um único documento e processo
  acessíveis, pelo mesmo índice/pesquisa/árvore usados por
  [`ies documento DOC`](documento-obter.md). A barra só autoriza seguir
  o link da identidade documental efetivamente observada. Ambiguidade de
  DOC entre processos, ausência de vínculo DOC↔NUP ou barra sem ação:
  recusar com a causa tipada, **nunca escolher o primeiro resultado**.
  Este comando não recebe um NUP oculto nem fabrica o vínculo.
- **Processo:** `BLOCO` precisa ser encontrado em **exatamente um** tipo
  apropriado entre **interno** e **reunião**, consultando as listas tipadas
  publicadas pela unidade ([`ies bloco listar --tipo`](bloco-listar.md)).
  Se ambos os tipos oferecerem o mesmo número sem desambiguação observável,
  recusar sem escrita. `NUP` é identificado por abertura independente.
  Bloco de assinatura não aceita `--processo`; interno/reunião não
  aceita `--documento`. Não inventar a categoria do bloco pelo número.
- O número de bloco não encontrado na **página observada** é
  `não encontrado nesta página`, não prova de inexistência global.
  Sem link único e contexto completo, a CLI recusa antes de escrever.

## Prévia e idempotência

Sem `--confirmar`, a CLI lê bloco, identidade do alvo, pertencimento
antes da operação e ação oferecida. Informa `tipo_bloco`,
`alvo`, `presente_antes` e a necessidade de inclusão, sem gravar.
O item já comprovadamente presente tem resultado idempotente
`alterado:false`, mesmo com `--confirmar`, sem POST/GET mutador.
O item **não observado** não equivale a ausente: não declarar prévia
de inclusão como se a lista estivesse vazia. Quando não se consegue
verificar pertencimento, recusar a confirmação com motivo verificável.

A leitura `ies bloco N` atualmente demonstra detalhe tipado de
**assinatura** ([contrato](bloco-obter.md)); ainda não demonstra detalhe
interno/reunião. A fatia de `--processo` deve completar essa leitura
funcional junto da inclusão. Não usar HTML de detalhe inventado como
prova nem fingir que o leitor atual já cobre todos os tipos.

## Confirmação e prova causal

Com `--confirmar`, conferir novamente o alvo, o tipo, a oferta da ação
e os controles da tela fresca; derivar a submissão estritamente do
formulário servido, preservando ocultos recebidos e recusando campos
editáveis não observados. Efetuar **uma** escrita oferecida e **nenhum
retry**. Não seguir redirecionamentos automaticamente. A aceitação HTTP
sem releitura não certifica a inclusão.

As duas transições Web são distintas e não devem ser modeladas por um
POST genérico:

- **Documento → assinatura:** o oráculo Todos usa a ação
  `bloco_escolher`, controles de documentos e seleção do bloco. A
  confirmação candidata é 302 com destino da própria ação, seguida de
  **releitura independente** do detalhe de assinatura, conferindo
  documento pela identidade durável, não apenas número ou posição.
- **Processo → interno/reunião:** o oráculo Todos usa o formulário do
  processo (`frmVisualizar`), campos de bloco e ocultos frescos. A
  confirmação candidata é 302 para a árvore, seguida de releitura do
  **detalhe tipado do bloco** que contenha o NUP exato. A associação
  processo↔bloco deve persistir nas consultas seguintes.

Essas formas são **hipóteses derivadas do Todos**, não transcrições
certificadas dos corpos originais das cassettes. A fatia começa pelos
cards e pelo confronto apropriados; se a transição não estiver gravada,
usar hipótese/fixture mínima e RED/GREEN, sem cassette-gate.

## Saída e erros

Sucesso ou prévia (saída 0) publicam somente dados observáveis,
sem URLs assinadas, senhas, cookies ou `infra_hash`:

```json
{"bloco": "12", "tipo_bloco": "assinatura",
 "alvo": {"documento": "0011505"},
 "confirmado": false, "alterado": false,
 "presente_antes": false, "presente_depois": null}
```

Para processo, `alvo` traz `{"processo": "NUP"}`. `null` significa
**não observado**. Com inclusão comprovada, `confirmado:true`,
`alterado:true`, `presente_depois:true` (releitura independente).
Se já presente, `alterado:false` e nenhuma nova escrita. `confirmado`
representa a solicitação do usuário; não é prova autônoma de efeito.
Na prévia, apresentar que a confirmação é necessária.

Causas observáveis são erros tipados preservados até a
fronteira (ADR 0005), sem catálogo/subcódigo novo: alvo ambiguamente
identificado, bloco de tipo inadequado, item já presente, ação ou
formulário não oferecido, recusa da instalação e leitura insuficiente.
Falha antes da escrita: saída 1, sem mutação. Após tentativa de
escrita sem prova suficiente: **saída 1** com um erro tipado de
efeito incerto (a confirmação, BDD-381-02, ainda não existe no código),
carregando o que foi observado e orientando nova consulta de leitura,
**nunca retry** (decisão [#681](https://github.com/franklinbaldo/ies/issues/681)).
Se a leitura posterior divergir, não afirmar sucesso por mera coincidência
do estado desejado sem aceitação causal da submissão.

## Critérios de implementação e provas

Fatiar por alvo, sem abrir duas PRs concorrentes para o mesmo estado:
primeiro `--documento` em assinatura, depois `--processo` em
interno/reunião, com um único contrato. Cada fatia contém CLI real,
anti-IES funcional com SQLite efêmero, prévia, confirmação e leitura
posterior em nova execução da CLI. O estado pertence ao banco, não a
HTML congelado. Registrar **separadamente**: (1) PASS/FAIL/NOT_RUN
sintético CLI↔anti-IES↔SQLite, (2) confronto independente de ambos
com cassettes originais, (3) live em treinamento autorizado. Ausência
de cassette/REA/live não impede implementar/integrar código sintético
com CI oficial verde.

**Orçamento de requisições:** declarar e testar por caminho realmente
publicado, sem fixar número fictício antes do primeiro E2E. Prévia não
envia POST mutador; confirmação envia no máximo uma mutação e uma
verificação independente. O GET de abrir processo pode ter efeito e
precisa de avaliação anterior ao ensaio live.

**Evidências de origem:** [bloco.incluir](../evidencias/bloco-incluir.md);
[cards UnB](../../cassettes/unb/bloco-incluir-anotar-retirar-unb--c3232a87e814.md)
e [PEN1 interno](../../cassettes/pen1/bloco-interno-processos-pen1--69795bb274d6.md);
[guia cassette-first](../../docs/anti-ies/ciclo-cassettes.md).
A existência dos cards não demonstra que as transições nem seus corpos
tenham sido capturados.
