---
type: CommandContract
title: Atribuir e retirar marcador de processo pela interface Web
operation: processo.marcar
command: ies processo marcar NUP --marcador ID#NOME|--retirar ID#NOME [--texto T] [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo marcar` (Refs #255)

A identidade é `id#nome`: o **id** decimal é a identidade observada do
marcador da unidade, e o **nome** confere sua semântica. Apenas a primeira
cerquilha é separadora. O IES rejeita nome vazio e controles e nunca
publica URL assinada, cookie ou `infra_hash`.

## Fatia funcional implementada

- Entrada `NUP` e `id#nome` inválida sai 2 **antes de ler a sessão**.
  Exige exatamente `--marcador` ou `--retirar`; `--texto` só com atribuição e respeita o limite de 250 caracteres do textarea observado. Aceita múltiplas linhas e não presume Latin-1; a codificação pertence ao formulário servido quando a escrita for implementada.
- Usa a leitura Web existente do processo, **sem andamentos e propriedades**,
  mas com os marcadores; exige a unidade publicada pela tela, ignorando
  o atalho que não permite conferi-la.
- Exige a lista completa: a contagem de linhas observadas deve ser igual
  ao `total_declarado`. `null`, link ausente ou tela não reconhecida
  **não** significam lista vazia.
- Se o id está presente e o nome confere, atribuir é idempotente: saída 0,
  `alterado: false`, **sem POST**.
- Se o id não consta da lista completa, retirar também é idempotente,
  saída 0, sem escrita. Se o mesmo id tem nome diferente, recusa.
- Se está presente e o pedido é retirar, produz **prévia** com a unidade,
  id, nome e `requisicoes` realmente contadas. `--confirmar` recusa
  `remocao_sem_formulario_observado`: **nenhum POST**.
- Se está ausente e o pedido é atribuir, recusa
  `catalogo_da_unidade_nao_observado`, pois não basta o marcador não
  existir **no processo** para provar que ele existe **na unidade**.

A prévia não reivindica a escrita como implementada. Se `--confirmar` for
pedido, a CLI explica **antes de abrir sessão ou rede** que a confirmação
não está disponível e orienta a executar sem `--confirmar` para consultar
uma prévia. A instrução é idêntica nas saídas humana e JSON; não promete
que a prévia tenha removido ou atribuído o marcador. O código não executa
JavaScript e não interpreta mensagem do SEI para classificação.

## Orçamento e sequência da fatia

`ies processo NUP --sem-andamentos --sem-propriedades` com exigência da
unidade e leitura de marcadores. Requisições variam conforme abertura e
cache, mas são contadas pelo leitor. Não ocorre requisição adicional para
prévia, idempotência ou recusa. `--confirmar` neste recorte faz **zero
requisições de escrita**. O teste `previa_de_retirada_e_confirmacao_bloqueada_usam_quatro_get_zero_post`
usa uma abertura fria sintética pelo Controle: **3 GET para abrir o
processo + 1 GET para a tela dos marcadores = 4 GET, 0 POST**, tanto
na prévia quanto na confirmação recusada. O transporte roteirizado
confere a lista exata de pedidos, sem rede real. A contagem de outros
caminhos de abertura e do índice quente precisa de novos casos.
Testes sintéticos também conferem idempotência, lista parcial,
identidade divergente e texto Latin-1 válido. O E2E da prévia de retirada
presente usa a CLI Rust real, HTTPS local, anti-IES e SQLite efêmero,
compara saída humana/JSON e consulta novamente os marcadores para
comprovar **estado inalterado e zero POST**. Até que o formulário de
remoção seja validado, `retirada_possivel` permanece `false`, mesmo
quando a presença do marcador está confirmada. Presença não é prova de
ação oferecida. Isto é apenas a primeira
metade do BDD-255-02: remoção com `--confirmar` continua pendente.
Entrada inválida não lê configuração.

## Inventário normativo de subcódigos

A coluna abaixo é o catálogo verificável usado pelo gate `subcodigos`.
Todas as recusas de escrita permanecem **sem POST** neste recorte. Enquanto
a escrita não for observada, `--confirmar` recusa
`escrita_ainda_fechada` antes de abrir a sessão.

| Subcódigo | Condição observável ou regra |
| --- | --- |
| `rotulo_do_marcador_sem_separador` | Rótulo sem `#` |
| `id_do_marcador_nao_decimal` | Identificador ausente ou fora da forma decimal |
| `nome_do_marcador_vazio` | Nome vazio após o separador |
| `nome_do_marcador_com_controle` | Caracteres de controle no nome |
| `lista_de_marcadores_parcial` | Linhas observadas diferentes do total declarado |
| `rotulo_diferente_do_marcador_lido` | Mesmo id observado com nome diferente |
| `catalogo_da_unidade_nao_observado` | Atribuição ausente sem catálogo da unidade |
| `remocao_sem_formulario_observado` | Confirmação de remoção sem contrato da escrita |
| `escrita_ainda_fechada` | `--confirmar` nesta fatia WIP: saída 1 antes de abrir sessão ou rede, 0 requisições |
| `marcadores_nao_lidos_para_escrita` | Leitura ausente ou não comprovada |
| `unidade_da_sessao_nao_publicada_para_marcar` | Unidade não conferida |
| `texto_do_marcador_longo` | Texto excede 250 caracteres |

## Falhas tipadas

Entrada (2): `numero_processo_vazio`, `numero_processo_controle`,
`referencia_processo_legada`, `marcador_nao_informado`,
`rotulo_do_marcador_sem_separador`, `id_do_marcador_nao_decimal`,
`nome_do_marcador_vazio`, `nome_do_marcador_com_controle`,
`texto_do_marcador_longo`.

Leitura/decisão (1): erros específicos da abertura do processo, além de
`marcadores_nao_lidos_para_escrita`,
`unidade_da_sessao_nao_publicada_para_marcar`,
`lista_de_marcadores_parcial`,
`rotulo_diferente_do_marcador_lido`,
`catalogo_da_unidade_nao_observado`,
`remocao_sem_formulario_observado`.

O `status_http` e a mensagem saneada do SEI são preservados pela
camada de leitura existente. A saída de prévia JSON traz
`nup`, `id_procedimento`, `unidade`, `marcador`, `acao`,
`confirmado`, `alterado: false`, `nada_a_fazer`,
`retirada_possivel` e `requisicoes`.

## Ainda faltam (sem promessa de escrita)

Observar na cassette e ao vivo os formulários de **catálogo da unidade,
atribuição e remoção** (incluindo URLs assinadas e resposta), testes RED
do POST único derivado de `formulario_web`, prévia de atribuição ausente,
reconciliação por releitura após envio, saída 3 no efeito incerto, replay e
validação live UnB. Somente depois disso habilitar `--confirmar`.
Escritas são autorizadas apenas em **treinamento**, com prévia, um único
envio e verificação por leitura, nunca retry.

Ver [estrutura observada](../../docs/como-o-sei-funciona/marcadores-do-processo.md),
[decisão](../../docs/decisoes/processo-marcar.md) e
[relatório live](../../docs/validacao-live/2026-11-05-processo-marcar.md).
