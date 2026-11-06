---
type: CommandContract
title: Acompanhamento Especial por prévia e confirmação observada
description: Inclusão confirmada; backend de retirada em WIP, CLI bloqueada até CI e live
operation: processo.acompanhar
command: ies processo acompanhar NUP [--grupo G] [--observacao T] [--deixar] [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo acompanhar NUP` — inclusão e retirada em validação (Refs #254)

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
Grupo ausente ou repetido é erro tipado. A lista também recusa duas
linhas com o mesmo grupo e IDs distintos: nesse estado, selecionar a
primeira linha não comprovaria idempotência nem identidade do acompanhamento.
O nome vazio usa a opção em branco servida; não se inventa seu valor.

`--deixar` seleciona uma linha pela identidade; na ausência de
`--grupo`, exige no máximo uma. A prévia exige ainda o link `acaoExcluir`
da própria linha, o formulário `frmGerenciarAcompanhamento` com
`hdnInfraItemId` único por `name` e `id` no DOM, oculto habilitado, e o destino `acompanhamento_excluir` publicado
pela função servida para o mesmo processo. O JavaScript não é executado; a função `acaoExcluir` só serve de prova quando está em um script clássico executável, sem `src` e sem `type=module` ou MIME inerte. Código presente apenas como texto em script de dados, comentário ou string não autoriza a operação. A assinatura precisa ser uma declaração global fora de bloco ou expressão de função; templates e prefixos não reconhecidos falham fechados. Sem qualquer prova, a operação recusa antes de escrever. Se nenhum
acompanhamento existe, a prévia informa ausência de efeito. `--observacao` não combina com
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
- **Retirada:** `--deixar` tem somente prévia com validação estrutural da oferta de exclusão; `--deixar --confirmar` deriva a exclusão da action assinada que a função
  `acaoExcluir` servida publicou e do formulário original da mesma
  resposta, com `hdnInfraItemId` reescrito pela operação. Emite no máximo
  um POST, nunca segue 302 e relê o gerenciador. Somente considera
  sucesso quando a identidade exata desaparece e todas as demais linhas
  permanecem idênticas. Se o gerenciador redirecionar ao cadastro após
  a retirada do último acompanhamento, faz um GET explícito ao cadastro
  e valida sua identidade antes de reconhecer ausência. Resposta sem
  aceite do mesmo processo, falha do POST, leitura inconclusiva ou
  alteração concorrente resultam em `efeito_incerto` (saída 3), sem retry.
  Esta trilha ainda carece de CI e live em treinamento nesta fatia.
- **Orçamento:** na bancada sintética sem índice quente, 3 GETs de
  abertura + 1 GET gerenciar + 1 GET cadastro + 1 POST + 1 GET
  de verificação = **7 requisições** para inclusão confirmada;
  a prévia usa **5 GETs e 0 POST**. Para retirada, os cenários
  sintéticos preveem **4 GET + 1 POST + 1 GET** de conferência,
  total de **6 requisições**; se a última retirada redirecionar ao
  cadastro, acrescenta um GET explícito (total **7**).
  Os testes novos foram publicados antes do miolo, mas o GREEN ainda
  não foi comprovado no CI. O custo de abertura real varia
  com o índice local. Nenhum retry.
- **Fonte:** observação histórica do Todos na UnB em 15/10/2026
  (`tests/test_processo_acompanhar_010.py` @323eb01), cassettes
  `processo-acompanhar-*` e suas demais variantes.
- Testes sintéticos de derivação e transporte integrados; evidência ao vivo e replay ainda pendentes: ver
  [relatório](../../docs/validacao-live/2026-11-05-processo-acompanhar.md).

Os erros possuem `subcodigo` estável:
`acompanhamento_nao_publicado_na_barra`,
`cadastro_de_acompanhamento_de_outro_processo`,
`grupos_de_acompanhamento_nao_publicados`,
`grupo_de_acompanhamento_nao_oferecido`,
`grupo_de_acompanhamento_ambiguo`,
`lista_de_acompanhamentos_nao_publicada`,
`linha_de_acompanhamento_invalida`,
`acompanhamento_para_remover_ambiguo` e
`exclusao_de_acompanhamento_nao_oferecida`, entre outros
em `src/processo_acompanhar.rs`.

Todos os URLs assinados e cookies ficam privados; nenhuma saída ou
fixture contém credencial real. Nenhuma operação é dirigida à produção.

## Diagnóstico de confirmação não comprovada

Se um POST foi tentado e a operação sai com código 3, o JSON mantém o
subcódigo histórico e acrescenta `requisicoes` (total de chamadas de
transporte tentadas na invocação, incluindo o POST e a releitura) e
`motivo`: `resposta_sem_aceite`, `releitura_indisponivel` ou
`estado_divergente`. Essas categorias são evidência limitada, nunca
prova de aplicação. Não incluem cookie, URL assinada ou corpo de resposta.
O campo `resposta_post` classifica apenas a resposta observada: `html_sem_redirecionamento`,
`redirecionamento_observado`, `validacao_infra_recusada`,
`status_http_recusado`, `rede_sem_resposta` ou `falha_da_submissao`.
Ele não expõe corpo, `Location`, URL assinada nem segredo, e não muda
o critério de aceite: estado posterior coincidente sem resposta admissível
continua saída 3. Uma recusa anterior à submissão publica `null` nesses
campos, sem inventar contagem ou classificação. Nenhum motivo autoriza
repetir a escrita.

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
| `confirmacao_de_acompanhamento_ainda_bloqueada` | Retirada confirmada impedida pela CLI até gates e live desta fatia |
| `exclusao_de_acompanhamento_nao_oferecida` | Link, função, oculto ou destino de exclusão da linha escolhida não comprovados |
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

## Bloqueio de segurança desta PR Draft

O backend de retirada já passou no
[CI 42159033526](https://github.com/franklinbaldo/ies/actions/runs/42159033526)
(head `297bab7`), mas **não houve replay específico do acompanhamento nem
prévia autenticada em treinamento**. O caminho público
`ies processo acompanhar --deixar --confirmar` permanece bloqueado
**antes da sessão e de qualquer requisição**. O backend de retirada não
deve ser invocado manualmente para escrever. Retirar o bloqueio somente
após replay aplicável e validação segura em treinamento.

Na inclusão, além da releitura exata da linha criada, o retorno 302
do POST precisa apontar ao gerenciador do **mesmo processo**. Um redirect
a outra ação ou processo gera efeito incerto (saída 3) sem retry,
mesmo quando uma leitura isolada mostra linha coincidente. Se houver
falha de transporte após POST possivelmente consumido, a releitura
independente ainda pode conciliar o efeito sem repetir a escrita.
Dois testes sintéticos publicados antes do código obtiveram RED:
[CI 42163834327](https://github.com/franklinbaldo/ies/actions/runs/42163834327)
registrou **2 falhas e 1.654 testes aprovados**. O GREEN da correção
é aferido separadamente no head final. A confirmação live autorizada
segue prévia → um único POST → releitura, com resíduo registrado;
se a sequência não puder ser completada com segurança, não escrever.


## Idempotencia de grupo observado na lista (#254)

Quando `--grupo G` nomeia exatamente um grupo ja presente na lista
completa do gerenciador, a propria leitura prova que o acompanhamento
existe na unidade. A CLI devolve `ja_estava=true`, `alterado=false`,
e, com `--confirmar`, `confirmado=true`, **sem abrir cadastro e sem
POST**, independentemente da observacao informada. Na bancada sintetica
sem indice aquecido, o orcamento e de **4 GETs, 0 POSTs**. Se a lista nao
comprova aquele grupo, a derivacao segura do cadastro segue obrigatoria:
nao se infere ausencia nem se cria URL de formulario.

A saída de inclusão idempotente publica o **ID** e a **observação da linha realmente
lida** no gerenciador, mesmo quando `--observacao` pede outro texto:
não ocorreu alteração e não se pode anunciar o valor solicitado como
estado existente. `id_acompanhamento` nesse caso é identidade **já observada**,
não evidência de escrita nova. A tabela deve ter seis colunas, com `Grupo` na segunda e
`Observação` na terceira, sem rótulos duplicados, e o cabeçalho precisa
preceder as linhas de dados. Outras disposições não observadas são recusadas
com `colunas_de_acompanhamentos_invalidas`, sem POST.
