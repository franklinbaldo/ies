---
type: SEIObservation
title: "Escritas em lote nas sete instalações de treinamento: PASS em nove comandos"
description: "Ensaio de 10/11/2026: processo criar, documento criar, assinar, ciência de documento e de processo, sobrestar, retomar, concluir e reabrir com saída 0 e releitura independente na UnB, PEN1-4, UFRJ e GO, depois de corrigir as telas reais; cancelar sem oferta em todas"
---

# Escritas em lote — sete instalações de treinamento, 10/11/2026

**Regra:** aprovação para produção exige ensaio em todas as instalações de
treinamento (decisão do proprietário de 10/11/2026; `src/origem_de_treinamento.rs`).
**Contas:** as públicas da [tabela autorizada](../interfaces/credenciais-de-treinamento.md);
sessões em pastas `0700` com arquivos `0600`, fora do repositório.
**Execução:** binário release desta PR mais gravação das respostas em
arquivos `0600` fora do repositório, apagados depois. Cada `--confirmar` foi
executado uma única vez, com trava `O_EXCL`. Uma saída 3 nunca foi repetida
sobre o mesmo alvo: a releitura conferiu o efeito, e o PASS veio de outro
alvo, com o código corrigido.
**Orçamento:** declarado 100 requisições de confirmação por instalação;
contadas pela CLI, de 73 a 81 por instalação, mais prévias e leituras de
diagnóstico.

## Resultado

| Comando | UnB | PEN1 | PEN2 | PEN3 | PEN4 | UFRJ | GO |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `processo criar` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `documento criar` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `documento assinar` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `documento ciente` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `processo ciente` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `processo sobrestar` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `processo retomar` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `processo concluir` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `processo reabrir` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `documento cancelar` | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |

PASS = saída 0 com verificação por leitura (ciência com a atividade na
Lista de Ciências; sobrestar/retomar `reconciliado_por_leitura: true`;
criação com o novo item único na árvore ou o `Location` do processo).

**NOT_RUN — cancelar:** nenhuma instalação publica "Cancelar" na barra do
documento assinado para a conta pública; a CLI recusou na prévia
(`cancelamento_nao_oferecido_na_barra`), sem POST. **Fora do ensaio:** a
anexação com `--confirmar` não está implementada (#270); acompanhar e
marcar só têm prévia.

## Defeitos corrigidos (teste RED antes de cada um)

| Tela real | Onde | Correção |
| --- | --- | --- |
| `exibirSeries('U')`: lista de tipos já completa | UnB, PEN, UFRJ | `filtro_de_todos` → `Ok(false)` |
| `function escolher(idSerie)` e nomes com `&nbsp;` | PEN, UFRJ, GO | os dois parâmetros; nome como texto |
| `Location` do POST criador com `#ID-<id>` | UnB | o fragmento sai antes de validar |
| Gerados paginados na conta compartilhada | PEN1 | o POST segue; sem fotografia completa, resposta incerta sai 3 |
| `cienciaProcessoAnexado` casava o prefixo | UnB | prefixo com `(` |
| `var linkCienciaProcesso = '…';` | PEN, UFRJ, GO | `urls_das_variaveis`, também no retomar |
| script em `<!--//--><![CDATA[//><!--` | UnB | fonte bruta da tag, além do texto |
| instruções antes da recarga com `id_atividade` | todas | recusa só desvio de fluxo |
| legenda "Lista de Ciências (1 registro):" | todas | aceita a contagem |
| `if (confirm(` ativo no retomar | UnB | confirm ativo ou comentado |
| `btnPesquisar` e rádio "S" já marcado | UnB, UFRJ, GO | não vão no POST |
| "Processo sobrestado na unidade <a> (atribuído para <a>)." | UFRJ, GO | lido como sobrestado |
| rótulo `Despacho 1407 (8446460)` | GO | numeração do tipo aceita |

A prévia da anexação também passou a funcionar na UnB (`<button>` de envio e
consulta AJAX em XML, `FalhaTransporte::TipoDeConteudoNaoXml`).

## Efeitos incertos (saída 3), conferidos por leitura

Antes das correções, estas confirmações gravaram, mas a CLI não provou o
efeito. Nenhuma foi repetida.

- UnB: `processo criar` (`23106.001912/2026-63`, criado); ciência de processo e
  documento no mesmo processo e no `23106.001913/2026-16` (registradas).
- PEN1–4, UFRJ e GO: ciência de processo e de documento da rodada 2
  (aceite do SEI 5 não reconhecido).
- UFRJ e GO: sobrestamento da rodada 2 (andamento "Sobrestamento" presente).
- GO: dois `documento criar` da rodada 2 (`8446443` e `8446460`, criados).

## Resíduos

Processos de teste criados: UnB `23106.001912/2026-63` e
`23106.001913/2026-16`; PEN1 `043493/2026-10`; PEN2 `98290.000148/2026-78`;
PEN3 `98390.000099/2026-45`; PEN4 `98490.000119/2026-50`; UFRJ
`99079.901798/2026-18`; GO `202600005006025`. Todos ficaram abertos e sem
sobrestamento, com despachos assinados de teste e ciências registradas. A
ciência de processo também ficou nos processos B da
[cobertura anterior](2026-11-10-cobertura-7-instalacoes.md).

## Resultado

**PASS nas sete instalações** para os nove comandos da tabela.
