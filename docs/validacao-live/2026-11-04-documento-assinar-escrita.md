---
type: SEIObservation
title: Ensaio live da escrita de documento assinar (PEN1 e UnB)
description: Primeira escrita ao vivo de ies documento assinar --confirmar, autorizada pelo proprietário em 04/11/2026, num Despacho criado pelo Todos para o ensaio na PEN1 5.0.4 e na UnB 4.0.9; PASS nos quatro passos das duas instalações, requisições contadas no relay CONNECT local
---

# Ensaio live da escrita de `documento assinar` — 04/11/2026

Executa o [plano da escrita](2026-11-04-documento-assinar.md) do
[contrato](../../contracts/cli/documento-assinar.md). Autorização do
proprietário em 04/11/2026, só para instalações de treinamento e só para
este plano. Binário `ies` construído do commit `1a4bd85` (sem build
nesta rodada); sessões e índices já existentes de cada instalação (nenhum
`ies login`: as duas sessões estavam válidas). Requisições contadas por um
relay CONNECT local (um túnel TLS por requisição do `ies`); 60 s ou mais
entre comandos na PEN1, 30 s ou mais na UnB; **nenhum retry**. A senha só
viajou pelo arquivo `0600` de `IES_CREDENCIAIS_FILE`. Nenhuma saída
registrada traz cookie, URL assinada, `infra_hash` ou senha: as duas
únicas URLs nas saídas são o endereço público de conferência de
autenticidade impresso no corpo assinado (sem `infra_hash`).

| Instalação | Documento assinado | Processo | 1 Prévia | 2 Escrita | 3 Leitura | 4 Prévia de novo |
| --- | --- | --- | --- | --- | --- | --- |
| PEN1 5.0.4 (`orgao1`, TESTE) | Despacho `0012327` (id 16863) | `08227.000048/2026-12` | **PASS** | **PASS** | **PASS** | **PASS** |
| UnB 4.0.9 (`aluno1`, Unidade1) | Despacho `0012128` (id 51707) | `23106.001850/2026-90` | **PASS** | **PASS** | **PASS** | **PASS** |

Nenhuma saída 3; nenhuma reconciliação (`reconciliado_por_leitura: false`
nas duas escritas).

## Documento do ensaio (Todos)

Nenhum documento sem assinatura do ensaio existia; cada um foi criado com o
Todos 0.24.255 (`uv run --offline --project /home/user/todos todos …`, HOME
isolado no scratchpad, sessão em arquivo, senha pelo ambiente do processo,
nunca em argumento). Só `documento criar`; nenhuma assinatura pelo Todos.
O Todos reaproveita a conexão (keep-alive): o relay conta **túneis**, não
requisições, para ele.

| Hora (UTC) | Instalação | Comando | Saída | Túneis | Resultado |
| --- | --- | --- | --- | --- | --- |
| 22:47:55 | PEN1 | `documento criar --processo 08227.000048/2026-12 --tipo Despacho --acesso publico --descricao "ensaio IES documento assinar 2026-11-04" --json` (prévia) | 0 | **não contados** | `executado: false`; o login e as leituras do Todos saíram **fora do relay** (abaixo) |
| 22:49:25 | PEN1 | o mesmo com `--confirmar` | 0 | 1 (enviado 9.830 B, recebido 54.459 B) | `documento: 0012327`, `id: 16863`, `verificado: true` |
| 22:55:00 | UnB | `documento criar --processo 23106.001850/2026-90 …` `--confirmar` | 0 | 1 (11.650 B / 121.303 B) | `documento: 0012127`, `id: 51706`, `verificado: true`, **na Unidade2** (abaixo) |
| 22:56:36 | UnB | o mesmo com `--unidade Unidade1` | 0 | 1 (12.607 B / 219.797 B) | `documento: 0012128`, `id: 51707`, `verificado: true`, na Unidade1 |

**Prévia do Todos fora do relay.** O invólucro passava `HTTPS_PROXY` para o
relay e `https_proxy=` vazio. O Rust do `ies` ignora a variável vazia, mas
o `urllib.request.getproxies()` do Python (usado pelo HTTPX) deixa a
minúscula vazia **remover** o proxy, e a prévia do Todos (que entrou no SEI:
login e leitura do processo, entre 22:47:58 e 22:48:03) saiu sem passar
pelo relay. Daí em diante as duas variáveis apontaram o relay. Nenhum
comando do `ies` foi afetado (todos contados).

**Despacho `0012127` órfão na UnB.** A sessão nova do Todos entrou na
Unidade2 (o processo está aberto na Unidade1 e na Unidade2; o índice do
Todos registrou `unidade: Unidade2`), e o documento nasceu lá. A prévia do `ies`, na sessão da
Unidade1, recusou com saída 1 (abaixo), o que está correto: o SEI não põe
"Assinar" na barra para um documento não assinado de outra unidade, e as
árvores da UnB gravadas antes publicam `onclick="assinarDocumento();"`
quando a ação existe. O passo de criação foi refeito com `--unidade
Unidade1` (`0012128`). O `0012127` ficou **sem assinatura e sem outra
operação** no processo `23106.001850/2026-90` (Unidade2), com a descrição
do ensaio.

## PEN1 5.0.4 — `08227.000048/2026-12`, Despacho `0012327`

| Hora (UTC) | Passo | Comando `ies` | Saída | Túneis = `requisicoes` | Campos | Veredito |
| --- | --- | --- | --- | --- | --- | --- |
| 22:50:32 | 1 | `documento --processo 08227.000048/2026-12 assinar 0012327 --cargo "Assessor(a)" --json` | 0 | 3 = 3 | `assinaturas: []`, `ja_assinado_pela_sessao: false`, `edicao: nao_assinado`, `assinante: Usuário Teste Orgão 1`, `cargo: Assessor(a)`, `cargo_da_tela: Agente Fiscalizador de Contrato`, 21 `cargos` com `Assessor(a)` | **PASS** |
| 22:51:38 | 2 | o mesmo com `--confirmar` | 0 | 5 = 5 | `confirmado: true`, `alterado: true`, `assinaturas_depois: [{nome: Usuário Teste Orgão 1, cargo: Assessor(a), unidade: TESTE}]`, `edicao_depois: assinado`, `reconciliado_por_leitura: false` | **PASS** |
| 22:52:45 | 3 | `documento --processo 08227.000048/2026-12 0012327 --sem-copia --json` | 0 | 2 = 2 | `rotulo: Despacho 0012327`, `versao: 1`; o corpo traz "Documento assinado eletronicamente por Usuário Teste Orgão 1, Assessor(a), em 04/11/2026, às 19:51" (Brasília, = 22:51 UTC) | **PASS** |
| 22:53:49 | 4 | o do passo 1 | 0 | 3 = 3 | `ja_assinado_pela_sessao: true`, `alterado: false`, `assinaturas` com o item do passo 2, `edicao: assinado`, `cargo_da_tela: Assessor(a)` | **PASS** |

Túneis da escrita (bytes enviados/recebidos): abertura 1.333/50.727 (GET
pelo link da árvore guardado), tela do documento 1.297/27.591, tela de
assinatura 1.300/23.568, **POST** 2.075/21.597, verificação 1.333/51.286
(árvore relida). É o orçamento do contrato com a abertura de 1 GET: 1 + 2 +
1 + 1 = 5. A prévia gastou 1 + 2 = 3.

## UnB 4.0.9 — `23106.001850/2026-90`, Despacho `0012128`

| Hora (UTC) | Passo | Comando `ies` | Saída | Túneis = `requisicoes` | Campos | Veredito |
| --- | --- | --- | --- | --- | --- | --- |
| 22:54:26 | escolha | `processo 23106.001850/2026-90 --sem-andamentos --sem-propriedades --sem-marcadores --json` | 0 | 2 = 2 | `aberto_em`: Unidade1 (atribuído a `aluno2`) e Unidade2; sessão válida | — |
| 22:55:42 | 1 (`0012127`) | `documento --processo 23106.001850/2026-90 assinar 0012127 --json` | 1 | 1 | `fonte_invalida` / `assinatura_nao_publicada_na_barra`, "nada foi assinado" (documento da Unidade2, acima) | — (recusa correta) |
| 22:57:14 | 1 | `documento --processo 23106.001850/2026-90 assinar 0012128 --json` | 0 | 3 = 3 | `assinaturas: []`, `ja_assinado_pela_sessao: false`, `edicao: nao_assinado`, `assinante: aluno1`, `cargo: null`, `cargo_da_tela: Diretor`, `cargos: [Administrador(a), Arquivista, Diretor]` | **PASS** |
| 22:57:48 | 2 | `documento --processo 23106.001850/2026-90 assinar 0012128 --cargo "Diretor" --confirmar --json` | 0 | 5 = 5 | `confirmado: true`, `alterado: true`, `assinaturas_depois: [{nome: aluno1, cargo: Diretor, unidade: Unidade1}]`, `edicao_depois: assinado`, `reconciliado_por_leitura: false` | **PASS** |
| 22:58:25 | 3 | `documento --processo 23106.001850/2026-90 0012128 --sem-copia --json` | 0 | 2 = 2 | `rotulo: Despacho 0012128`, `versao: 1`; o corpo traz "Documento assinado eletronicamente por aluno1, Diretor, em 04/11/2026, às 19:57" | **PASS** |
| 22:58:59 | 4 | `documento --processo 23106.001850/2026-90 assinar 0012128 --cargo "Diretor" --json` | 0 | 3 = 3 | `ja_assinado_pela_sessao: true`, `alterado: false`, `assinaturas` com o item do passo 2, `edicao: assinado` | **PASS** |

Túneis da escrita: abertura 1.139/69.037, tela do documento 1.103/35.814,
tela de assinatura 1.106/24.591, **POST** 1.810/22.605, verificação
1.139/70.004 (5 = orçamento). A recusa do `0012127` gastou só a abertura
(1 GET), como o contrato prevê para a recusa da árvore.

**Desvio do passo 1 na UnB.** A prévia rodou sem `--cargo`: nenhum cargo da
UnB estava registrado à mão (as cassetes do Todos não estão no disco). A
prévia publicou os três `cargos`, e a escrita usou `Diretor` (a opção
`selected`), conferido de novo pela tela antes do POST. O passo 4 levou o
`--cargo`.

## O que o ensaio confirma

- A escrita na PEN1 5.0.4 e na UnB 4.0.9 (as instalações das duas formas
  do link de assinar no plano, `var linkAssinarDocumento` e a função de
  três linhas; a saída não diz qual forma foi lida), com o POST
  `frmAssinaturas` derivado e a resposta reconhecida como a esperada
  (`//se realizou assinatura`), o que se infere de nenhuma
  `falha_do_envio` e `reconciliado_por_leitura: false`.
- A verificação pela caneta da árvore relida: um item novo, igual ao
  assinante da tela, ao cargo pedido e à unidade da sessão. A edição passou
  de `nao_assinado` a `assinado`.
- A leitura independente pelo conteúdo (`documento DOC`) mostra o bloco de
  assinatura com o mesmo nome e cargo.
- A prévia depois da escrita sai 0 com `ja_assinado_pela_sessao: true`,
  sem POST.
- Contagens: `requisicoes` igual aos túneis em todos os comandos do `ies`.

## Não verificado

- Requisições do Todos: só túneis (keep-alive), e a prévia da PEN1 ficou
  fora do relay.
- Saída 3 e `assinatura_nao_aplicada` (senha recusada) ao vivo: não
  provocadas (a escrita não se desfaz).
- Documento com dois assinantes e assinatura em documento de outra unidade:
  não exercitados (o `0012127` só mostrou a recusa da barra).
- Outras instalações (PEN2–4, UFRJ, GO): fora desta autorização.
