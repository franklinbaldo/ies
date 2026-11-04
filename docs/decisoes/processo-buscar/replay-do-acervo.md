---
type: ImplementationDecision
title: Replay de conformidade do acervo público pelas peças de processo buscar
operation: processo.buscar
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de processo.buscar (Refs #58); um teste ignorado do próprio crate passa cada resposta gravada do Controle e da tela de Pesquisa (protocolo_pesquisar, GET e POST) dos 52 cards públicos pelas mesmas funções de ies processo buscar, offline, e classifica lido, recusado ou não aplicável; sem superfície nova na CLI e sem rede no CI
selected_strategy: teste-ignorado-no-crate
alternatives: teste-ignorado-no-crate,subcomando-oculto-de-replay,corpos-gravados-como-fixture-do-ci
evidence_level: replay-offline-do-acervo-publico-e-testes-sinteticos
live_validation: not-run-replay-offline-nao-substitui-ensaio-live
---

# Replay de conformidade do acervo — processo buscar

Apêndice da [decisão de `processo.buscar`](../processo-buscar.md), com o
mesmo desenho do [replay de `processo NUP`](../processo-obter/replay-do-acervo.md).
Contrato: [processo buscar](../../../contracts/cli/processo-buscar.md);
resultado no [relatório](../../validacao-replay/2026-11-04-processo-buscar-acervo.md).

A pergunta: as peças do comando, escritas a partir de capturas da UnB e
de ensaios em UnB, GO e PEN1, leem as telas de Pesquisa gravadas nas
**sete** instalações? Antes, o ensaio offline sobre as cassettes era um
teste temporário não versionado.

## Alternativas consideradas

- **teste-ignorado-no-crate** (escolhida): módulo `cfg(test)` em
  `src/pesquisa_web/acervo/`; `replay_do_acervo_inteiro` é `#[ignore]`,
  lê os ZIPs de `IES_ACERVO_CACHE` pela conferência de
  `cassette::acervo` (SHA-1 do ZIP, SHA-256 do membro, schema, host) e
  escreve em `IES_ACERVO_SAIDA`. O CI roda os testes sintéticos.
- **subcomando-oculto-de-replay**: superfície de CLI nova para uma
  ferramenta de conformidade. Rejeitada.
- **corpos-gravados-como-fixture-do-ci**: os corpos trazem `infra_hash` e
  URLs assinadas. Rejeitada.

## Comparação

| Critério | Teste ignorado | Subcomando | Fixture bruta |
| --- | --- | --- | --- |
| Superfície pública nova | não | sim | não |
| Mesmas funções do comando | sim | sim | sim |
| Credencial gravada no Git | não | não | **sim** |
| Rede no CI | não | não | não |

## Decisão

A sequência de cada cassette segue o fluxo do comando, e o que a
gravação não estabelece é `nao_aplicavel`:

1. **Controle** (`procedimento_controlar`, GET): transporte,
   `unidade_do_controle` e `link_da_pesquisa` sobre a URL servida. Só um
   Controle lido inteiro fornece a unidade da etapa seguinte.
2. **Tela** (`protocolo_pesquisar`, GET): `derivar_pesquisa_de_protocolo`
   com essa unidade e o termo de prova `0`. O corpo dos POSTs gravados
   não existe nas cassettes, e o termo não muda a resposta já gravada.
3. **Envio** (`protocolo_pesquisar`, POST após a tela): a resposta gravada
   (status, `Content-Type`, `Location`, corpo) entra no lugar do POST em
   `enviar_submissao_via`, e `interpretar_envio` a classifica como faz o
   comando (302 conferido contra a URL derivada, validação da Infra,
   marcador, lista, "sem resultado").
4. **Plano da página 2**, para cada lista lida: `planejar_pagina(…, 2)` e
   `derivar_pagina_da_pesquisa` sobre a tela de resultado, sem envio.
   Uma lista de uma página só (`pagina_fora_do_intervalo` com 1 página)
   conta como lida.

A tela de resultado publica o rádio `rdoPesquisarEm` marcado. Com
`#optDocumentos` marcado, as linhas não são de processo (o comando só
envia `P`): a lista é `lista_da_pesquisa_em_documentos`, e só o aviso
de "sem resultado" é lido (`sem_resultado_em_documentos`). Um POST feito
a partir da tela de resultado (paginação ou nova pesquisa, indistinguíveis
sem o corpo) é `post_da_tela_de_resultado_sem_corpo_gravado`.

O replay achou uma forma nova em Goiás: na pesquisa de processos, a
linha cujo acerto é um documento leva `id_documento` no link do processo
e dois `a.protocoloNormal` de `documento_visualizar`. `ler_resultados`
passa a aceitar esses links quando o `id_documento` é o mesmo do link do
processo (`src/pesquisa_web/documento_da_linha.rs`); qualquer outra
forma é `link_do_documento_da_linha_diferente_do_processo`. A saída
segue publicando só o número do processo.

## Evidências verificáveis

- [Contrato](../../../contracts/cli/processo-buscar.md) e
  [matriz Todos](../../../contracts/evidencias/processo-buscar.md).
- Testes sintéticos: `pesquisa_web::acervo::tests` (classificação) e
  `pesquisa_web::tests_arvore` (linha com acerto em documento);
  [superfície da CLI](../../../tests/processo_buscar_web.rs).
- [Relatório do replay](../../validacao-replay/2026-11-04-processo-buscar-acervo.md).

## Limitações e revisão

- Replay offline não é ensaio live: sem corpo de POST gravado, o termo e
  a página pedida não são conhecidos, e nenhum POST de página de
  pesquisa **de processos** foi gravado (as quatro cassettes
  `pesquisa-paginas-*` pesquisam documentos).
- Nenhuma resposta "sem resultado" de pesquisa de processos está no
  acervo; o aviso só aparece em pesquisas de documentos (UFRJ, PEN3).
- `processo-busca-go#4` publica 9 linhas para a barra "Exibindo 1 - 10
  de 966" e segue recusada (`linhas_da_tabela_diferentes_da_faixa_da_barra`).
  A causa não está na tela (hipótese: processo contado e não exibido
  por restrição de acesso); não se infere nada da linha que falta.
- Rever se o acervo ganhar cassettes de pesquisa de processos com
  paginação ou "sem resultado", ou se outra instalação publicar outra
  forma de linha.
