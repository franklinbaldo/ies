---
type: Guide
title: Matriz de derivação dos comandos ies unidade
description: Linhas da matriz de derivação para listar-processos, colunas, recebidos, gerados, atual e trocar, com as decisões e pendências da caixa da unidade
---

# Matriz de derivação — `ies unidade *`

Parte da [matriz de derivação da CLI](matriz-derivacao-cli.md), com os
mesmos estados, a mesma regra de priorização e o mesmo critério de
contagem: **6 caminhos de unidade entre os 20 da main**, incluindo
`trocar` (#205), integrado em #206. Separada por tamanho: a caixa
(#57, feita) concentra as decisões de 02/11/2026. Base verificada:
`main@52be0e1`, depois de #193 e #206.

| Comando existente | Estado na main | Já demonstra | Falta implementar, sem novo comando/flag | Referência |
| --- | --- | --- | --- | --- |
| `ies unidade listar-processos [--pagina N]` | **PARCIAL WEB** (decisão do proprietário, 02/11/2026) | Visão D com colunas, categoria observada na R do mesmo GET e `colunas_ausentes`; troca R→D sem volta ou D direta, sem restaurar visão nem página (a sessão fica na D); 1 GET + 0 a 2 POST, 1 GET em regime estável | [live 04/11](validacao-live/2026-11-04-unidade-paginacao.md): PASS UnB, PEN1, PEN3, PEN4 (D com 3 páginas), UFRJ (D de 1 página, após correção); GO 5.0.3 PASS na [3ª tentativa](validacao-live/2026-11-04-unidade-paginacao-go-pen2.md) (página de 10, R→D, D lembrada, D→R; sem 403 a 90 s); PEN2 NOT_RUN ([2ª tentativa](validacao-live/2026-11-04-unidade-paginacao-go-pen2.md): login com 403 do Cloudflare na PEN2 e interrompido pelo proxy em GO); [replay](validacao-replay/2026-11-04-unidade-listar-processos-acervo.md) | [contrato](../contracts/cli/unidade-listar-processos-visao-detalhada.md) |
| `ies unidade colunas [--definir todas\|LISTA [--confirmar]]` | **PARCIAL WEB** (`--definir`: escrita remota) | 12 opções do catálogo do Nível de detalhe com `disponivel` (a UnB 4.0.9 publica 11) e `marcada`; D: 2 GET; R: 2 GET + 1 POST D, e a sessão fica na D. `--definir`: prévia sem salvar; `--confirmar`: 1 POST de salvar + 1 GET de verificação, reconciliação e saída 1; resposta do salvar esperada só com o sinal de gravação (`inicializar` `'1'=='1'`) | Live PASS PEN1 e UnB 02/11, UnB 04/11; PEN1 04/11 FAIL (saída 1, sem efeito, causa não registrada: [análise](validacao-live/2026-11-04-definir-colunas-pen1.md)); PEN4 04/11 **PASS** (3 → 12 colunas, 1 POST com sinal de gravação, sem reconciliação, `colunas_ausentes` vazia: [relatório](validacao-live/2026-11-04-definir-colunas-pen4.md)); PEN3 5.0.4 **PASS** (12 colunas, 1 POST, sem reconciliação: [relatório](validacao-live/2026-11-04-definir-colunas-pen3.md)) | [leitura](../contracts/cli/unidade-colunas.md), [escrita](../contracts/cli/unidade-definir-colunas.md) |
| `ies unidade recebidos [--pagina N]` | **PARCIAL WEB** (decisão do proprietário, 02/11/2026) | Lista Recebidos: página observada com total e faixa; `--pagina N` com 1 GET + até 1 POST; sessão na D: troca para R e fica nela (+1 POST) | [live 04/11](validacao-live/2026-11-04-unidade-paginacao.md): PASS UnB, PEN1, PEN3, PEN4, UFRJ (página lembrada); GO 5.0.3 PASS na [3ª tentativa](validacao-live/2026-11-04-unidade-paginacao-go-pen2.md) (página de 10, R→D, D lembrada, D→R; sem 403 a 90 s); PEN2 NOT_RUN ([2ª tentativa](validacao-live/2026-11-04-unidade-paginacao-go-pen2.md)); lista vazia sem tabela de GO lida no replay; [replay](validacao-replay/2026-11-04-unidade-listar-processos-acervo.md) | [contrato](../contracts/cli/unidade-recebidos-gerados.md) |
| `ies unidade gerados [--pagina N]` | **PARCIAL WEB** (idem) | Lista Gerados, mesma regra e mesmo transporte | Idem | [contrato](../contracts/cli/unidade-recebidos-gerados.md) |
| `ies unidade atual` | **PARCIAL WEB** | Rótulo da unidade na página de Controle identificável | Live PASS UnB, UFRJ, PEN1, PEN3, PEN4 e GO (2ª execução) em [02/11](validacao-live/2026-11-02-leitura-controle.md); PEN2 NOT_RUN (503). Não infere permissão nem troca unidade | [contrato](../contracts/cli/unidade-atual.md) |
| `ies unidade trocar SIGLA [--confirmar]` | **PARCIAL WEB** (#205, escrita de estado da sessão) | Prévia pela tela `infra_trocar_unidade` do `lnkInfraUnidade` do Controle (2 req.; já na unidade, 1); `--confirmar`: lock do login, 1 POST com `selInfraUnidades`, os dois 302 (`http://` no GO, hash de 192) até o Controle novo (5 req.), URL da sessão e marcador regravados, links assinados da instalação apagados do índice; desvio depois do POST sai com 1 | Live **PASS** UFRJ 4.1.5 05/11 (PR-P → PR-A → PR-P, 2/5/5 req. iguais aos túneis, resíduo do #173 limpo: [relatório](validacao-live/2026-11-05-unidade-trocar.md)); UnB, GO e PEN NOT_RUN; [replay](validacao-replay/2026-11-05-unidade-trocar-acervo.md); PEN sem gravação de troca | [contrato](../contracts/cli/unidade-trocar.md), [decisão](decisoes/unidade-trocar.md) |

**#57 (página observada e `--pagina N`; em main):** `unidade
listar-processos` deixa de recusar caixa paginada: devolve a página que a
tela mostra com `total_declarado` e `faixa_observada` por tabela, zero POST;
`unidade recebidos|gerados --pagina N` (antes `--categoria T --pagina N`)
faz no máximo um POST do paginador da tabela,
com efeito `PaginaOuFiltroLembrado` e sem restauração. Live PASS em
04/11/2026 (linhas acima). Inventário das outras leituras paginadas, ainda fora da
[regra geral](../contracts/cli.md): `replay --numero-processo` recusa
página parcial (`paginas_do_controle_nao_obtidas`) porque ausência numa
página não prova ausência na caixa (`processo NUP --resumo`,
`--resultado-pesquisa-file` e `--historico-privado` foram aposentados em
04/11/2026); a lista de versões de documento só
aceita legenda igual às linhas. Alinhar cada uma exige evidência da
legenda e, para `--pagina N`, do paginador daquela tela.

**Decisão do proprietário (02/11/2026), exceção explícita à regra de
priorização:** Recebidos e Gerados são listas independentes do SEI
(paginador, total, faixa e página lembrada próprios). Por isso ganham
os subcomandos `ies unidade recebidos` e `ies unidade gerados`, os únicos
com `--pagina N`; `listar-processos` devolve o array das duas listas e
perde `--categoria` e `--pagina`, sem alias. Não é capacidade nova da
operação `unidade.listar_processos`: mesmo GET, mesmo POST e mesmas
recusas ([decisão](decisoes/unidade-listar-processos.md)). A exceção não
autoriza outros verbos.

**Visão detalhada por padrão (decisão do proprietário, 02/11/2026; em
main):** `listar-processos` lê a visão D (uma lista com colunas e a
categoria observada por processo), troca de visão sem restaurar (a sessão
fica na D; decisão do proprietário, 02/11/2026: restaurar desperdiçava
requisições), e volta a ter `--pagina N` (página da D, que fica lembrada); `--pagina-arquivo` sai. `unidade
colunas` lê o Nível de detalhe sem salvar. A
[decisão](decisoes/unidade-listar-processos.md) registra a exceção à regra
de não criar verbos para `colunas`, que integra a mesma operação.
Live: ver as linhas da tabela.

**Escrita das colunas (decisão do proprietário "B", 02/11/2026; em
main):** `unidade colunas --definir todas|LISTA [--confirmar]` é a
operação `unidade.definir_colunas`, a primeira **escrita** remota do IES: a
opção pertence ao comando já exposto (não é verbo novo) e substitui a
alternativa de salvar e restaurar a cada listagem. Sem `--confirmar`, prévia
com os campos do POST e zero escrita; com ele, um POST de salvar e um GET de
verificação, reconciliação por leitura e saída 1 para escrita não confirmada; vinda
da R, a sessão fica na D. `listar-processos` passa a sugerir
`--definir todas` para as `colunas_ausentes`; `ies setup` não aplica nada.
Offline com telas sintéticas de 12 e 11 opções; live: ver a linha de
`colunas` acima ([decisão](decisoes/unidade-definir-colunas.md),
[roteiro](validacao-live/roteiro-definir-colunas.md)).

## Consolidação pós-#206

A troca de unidade é um sexto caminho próprio, não uma flag das listas:
`ies unidade trocar SIGLA [--confirmar]` abre a tela de seleção
publicada pelo Controle, prévia sem escrita e, ao confirmar, POST,
retornos conferidos e atualização da sessão; **PASS live UFRJ**
([ensaio](validacao-live/2026-11-05-unidade-trocar.md)),
**NOT_RUN** UnB, GO e PEN.

As leituras de unidade participam do **relogin único** quando há
recusa tipada de sessão (#200), sem repetir a escrita e sem relogin
nas prévias. O [ensaio PEN4](validacao-live/2026-11-05-relogin.md)
registra a recuperação e a mudança de geração; expiração natural
segue `NOT_RUN`. Nenhum `PASS` de CI supre essa lacuna.
