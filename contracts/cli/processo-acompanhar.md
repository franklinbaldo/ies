---
type: CommandContract
title: Acompanhamento Especial por prévia e confirmação observada
description: Prévia de inclusão e retirada; confirmação causal com POST único, restrita ao treinamento para retirada
operation: processo.acompanhar
command: ies processo acompanhar NUP [--grupo G] [--observacao T] [--deixar] [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo acompanhar NUP` — prévia e confirmação por leitura (Refs #254, #348)

O comando abre a árvore do processo e confere a unidade da sessão.
Lê o link assinado `acompanhamento_gerenciar` publicado na barra da raiz.
O GET pode retornar 302 para `acompanhamento_cadastrar` quando não existe
acompanhamento naquela unidade; este redirecionamento é **dado**, não
seguido automaticamente, e o destino é verificado pelo processo, ação,
origem e assinatura antes de fazer um GET explícito.

Com acompanhamentos, a página publica a tabela com resumo
`Tabela de Acompanhamentos.` e `btnAdicionar`, cuja URL vem da própria
tela. A lista incompleta não prova ausência. O grupo é escolhido pelo
**nome exato** entre as opções habilitadas de `selGrupoAcompanhamento`.
Grupo ausente ou repetido é erro tipado. O nome vazio usa a opção em
branco servida; não se inventa seu valor.

`--deixar` seleciona uma linha pela identidade; na ausência de
`--grupo`, exige no máximo uma. Se nenhum acompanhamento existe,
a prévia informa ausência de efeito. `--observacao` não combina com
`--deixar`. Uma inclusão de grupo já presente informa que nada deve
ser enviado, independentemente da observação.

## Estado desta fatia

- **Prévia:** implementada com leituras reais da interface Web. A saída
  informa `numero`, `unidade`, `operacao`, `grupo`, `observacao`,
  `id_acompanhamento` quando houver, `ja_estava`, `confirmado`,
  `alterado`, `origem:web` e `requisicoes`.
- **Inclusão `--confirmar`:** a tela servida é validada e dela é
  derivada exatamente uma submissão `acompanhamento_cadastrar`, com
  `selGrupoAcompanhamento` oferecido, `txaObservacao`, ocultos e botão
  observados. Um POST no máximo, sem retry e sem seguir 302. Releitura
  explícita do gerenciador prova uma única linha nova com grupo e
  observação pedidos, preservadas as linhas anteriores. Falha no POST
  também exige releitura: se a alteração não puder ser comprovada,
  saída **3** (`acompanhamento_nao_confirmado_por_leitura`, sem retry).
  A inclusão já existente resulta em sucesso sem POST.
- **Retirada `--deixar --confirmar`:** restrita a instalações públicas
  de treinamento e ao anti-IES (`sei.test`). A oferta da linha e a action
  assinada `acompanhamento_excluir` são reconhecidas na própria tela
  servida. O envio reescreve somente `hdnInfraItemId`, executa **um POST**
  e faz GET novo do gerenciador: só há confirmação quando exatamente a
  identidade solicitada desapareceu e todas as outras foram preservadas.
  Grupo solicitado já ausente é no-op sem POST. Resposta/estado incerto
  produz saída **3**, sem retry. Redirecionamento 302 após excluir a
  última linha não comprova lista vazia sozinho e permanece incerto.
  A comprovação aqui é sintética (CLI ↔ anti-IES ↔ SQLite); confrontos
  semânticos independentes com cassettes e treinamento/live são pendentes.
- **Orçamento:** na bancada sintética sem índice quente, a inclusão
  confirmada usa 3 requisições de abertura + GET gerenciar + GET cadastro
  + POST + GET de verificação = **7 requisições**. Retirada de uma linha
  com outras remanescentes usa 3 requisições de abertura + GET gerenciar
  + POST + GET de verificação = **6 requisições**. O custo de abertura
  real varia com o índice local; nunca há retry.
- **Fonte:** observação histórica do Todos na UnB em 15/10/2026
  (`tests/test_processo_acompanhar_010.py` @323eb01), cassettes
  `processo-acompanhar-*` e suas demais variantes.
- Testes sintéticos de derivação e transporte integrados; evidência ao vivo e replay ainda pendentes: ver
  [relatório](../../docs/validacao-live/2026-11-05-processo-acompanhar.md).

Os erros, por variante (a saída pública traz só a mensagem):
`acompanhamento_nao_publicado_na_barra`,
`cadastro_de_acompanhamento_de_outro_processo`,
`grupos_de_acompanhamento_nao_publicados`,
`grupo_de_acompanhamento_nao_oferecido`,
`grupo_de_acompanhamento_ambiguo`,
`lista_de_acompanhamentos_nao_publicada`,
`linha_de_acompanhamento_invalida`,
`acompanhamento_para_remover_ambiguo`, entre outros
em `src/processo_acompanhar.rs`.

Todos os URLs assinados e cookies ficam privados; nenhuma saída ou
fixture contém credencial real. Nenhuma operação é dirigida à produção.

## Inventário de erros tipados

A coluna literal abaixo é normativa e verificável pelo gate de subcódigos.

| Subcódigo | Condição |
| --- | --- |
| `acompanhamento_nao_confirmado_por_leitura` | A releitura não demonstrou a alteração exata; incerto |
| `acompanhamento_nao_publicado_na_barra` | A barra do processo não oferece Acompanhamento Especial |
| `acompanhamento_para_remover_ambiguo` | Mais de uma linha satisfaz a escolha de remoção |
| `barra_de_acompanhamento_fora_da_forma` | Barra fora do formato observado |
| `barra_de_acompanhamento_repetida` | A árvore publica mais de uma barra para a raiz |
| `cadastro_de_acompanhamento_de_outro_processo` | Identidade do processo divergente no cadastro |
| `cadastro_de_acompanhamento_nao_publicado` | Formulário de cadastro ausente |
| `colunas_de_acompanhamentos_invalidas` | Cabeçalho da lista incompleto ou repetido |
| `formularios_de_acompanhamento_repetidos` | Mais de um formulário para a operação |
| `grupo_de_acompanhamento_ambiguo` | O nome informado corresponde a mais de uma opção |
| `grupo_de_acompanhamento_nao_oferecido` | Grupo solicitado não é oferecido na tela |
| `grupo_ja_acompanhado_no_processo` | O grupo já aparece na lista anterior |
| `grupos_de_acompanhamento_nao_publicados` | Select de grupos ausente ou sem opções |
| `html_de_acompanhamento_invalido` | HTML ilegível ou acima do limite |
| `id_de_acompanhamento_repetido` | A tabela repete a identidade de uma linha |
| `linha_de_acompanhamento_invalida` | Colunas ou checkbox de uma linha fora da forma |
| `link_de_acompanhamento_de_outro_processo` | Link assinado da barra indica outro processo |
| `link_de_acompanhamento_nao_reconhecido` | Link da barra não segue a forma permitida |
| `links_de_acompanhamento_repetidos` | Mais de um link de acompanhamento na barra |
| `lista_de_acompanhamentos_nao_publicada` | Não existe tabela comprovando a lista |
| `listas_de_acompanhamentos_repetidas` | Mais de uma tabela de acompanhamento |
| `opcoes_de_grupo_de_acompanhamento_invalidas` | Option sem valor válido |
| `origem_do_acompanhamento_nao_autorizada` | Origem da sessão inválida |
| `pedido_de_acompanhamento_invalido` | Combinação de opções inválida |
| `processo_sem_acompanhamento` | Não há linha a selecionar para remoção |
| `redirecionamento_de_acompanhamento_inesperado` | Redirect não comprovado para o cadastro da mesma origem |
| `tela_de_acompanhamento_inesperada` | Formato ou controle da tela divergente |
| `unidade_do_acompanhamento_nao_publicada` | Unidade da tela não está publicada |
