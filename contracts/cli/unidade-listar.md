---
type: CommandContract
title: Lista local das unidades do SEI
operation: unidade.listar
command: ies unidade listar [--atualizar]
effect: read
implementation: partial
---

# Unidade listar (#244)

**Implementação:** sem `--atualizar`, lê o snapshot integral já
guardado no índice SQLite (esquema 3), sem enviar requisições ao SEI.
A identidade é a origem HTTPS autorizada e o ID da unidade ativa
da URL privada do Controle, ambos lidos localmente e sem publicar
a URL ou o cookie. A lista só pertence à unidade e instalação exatas.

`--json` retorna `origem: "indice"`, `lida_em` (época Unix),
`total`, `unidade_ativa`, `unidades` (cada uma com `id`, `sigla`,
`descricao`, `orgao`) e `requisicoes: 0`.
Sem JSON, mostra a contagem e as linhas com esses quatro campos.

**Cache ausente ou atualização solicitada:** o IES inicia a coleta
integral pela lupa Web, com orçamento documentado abaixo. Falhas de
rede, página ou integridade são recusas tipadas: não se apresenta o
snapshot anterior como se fosse atualizado. Se o índice estiver
desligado, a coleta Web continua e avisa que não houve persistência.
`--unidade` explícita é recusada com
`UnidadePedidaNaoVerificavelOffline`: a CLI não finge ter
verificado no SEI a sigla exibida. Nenhum caso faz retry.

A integridade, a consulta SQL pública somente leitura e a
migração estão em [tabela de unidades](../../docs/arquitetura/0003-indice-local-unidades.md).
Não implica que o snapshot ainda seja o mais recente.
[Decisão](../../docs/decisoes/unidade-listar.md) e
[evidência local](../evidencias/unidade-listar.md).


## Leitura Web integral e falhas observáveis

Quando não há snapshot ou foi solicitado `--atualizar`, o comando abre
o Controle, navega para a listagem de grupos de envio e para a tela
Novo, segue a lupa `unidade_selecionar_envio_processo`, faz o GET
inicial da lupa e **um POST para cada página**. O orçamento máximo
da lupa é `1 GET + N POST`, mais os três GETs de navegação.
A lista só é gravada se todas as páginas, índices, totais e IDs
conferirem. Cada resposta POST deve ainda confirmar que os filtros de
sigla e descrição estão vazios e que o órgão efetivamente selecionado
é `Todos` (valor vazio, nunca `null`). Uma lista filtrada com total
internamente coerente **não** constitui um snapshot integral. O resultado avisa `origem: "sei"`, o número de
`requisicoes` e `persistida_no_indice` (false quando o índice
estiver indisponível). Sem índice, a leitura retorna sem persistir.

**Orçamento também nas recusas:** quando a coleta Web falha, a saída
JSON inclui `requisicoes` com o número de tentativas GET e POST até a
recusa (inclusive a que falhou). A saída humana mostra a mesma contagem.
Uma recusa antes do primeiro HTTP publica `requisicoes: 0` quando a
coleta Web chegou ao validador. A contagem não inclui login anterior
nem revela URL assinada, cookie ou credenciais. O erro tipado e o
`status_http`, quando observado, são preservados. Uma tentativa falha
não provoca retry. Falhas de pré-condições locais, anteriores ao
início da coleta, mantêm o contrato geral de erro local.

O replay integral da lupa GO (SEI 5.0.3) mostrou os botões
`btnPesquisar` sem `name`; a operação os confere no formulário
servido e, nesse caso, não inventa um par `btnPesquisar` no POST.
A regra é local à lupa e conserva a recusa de formulários
não reconhecidos. A evidência é offline, sem atestar live.

Erros tipados desta operação:
- `lupa_transporte_recusado`: falha de transporte ou status na navegação/consulta;
- `FalhaDaLupa::LinkAusente`: ação obrigatória não foi publicada;
- `FalhaDaLupa::LinkAmbiguo`: links conflitantes para o mesmo destino;
- `FalhaDaLupa::LinkForaDaInstalacao`: link fora da origem permitida;
- `FalhaDaLupa::CharsetInesperado`: MIME não permite codificação segura;
- `FalhaDaLupa::FormularioInesperado`: controles/action da página não conferem;
- `ErroLupa::TabelaForaDaForma`: linha, tabela, cabeçalho ou paginação inválida;
- `ErroDasUnidades::LupaSemListaIntegral`: totais, páginas ou IDs não conferem;
- `FalhaDaLupa::ExcessoDePaginas`: número de páginas não foi encerrado.

A lista local é um retrato da leitura, não uma garantia de atualidade.

## Tabela normativa de recusas da lupa

| Erro ou `onde` (stderr) | Condição |
| --- | --- |
| `FalhaDaLupa::CharsetInesperado` | Codificação da tela não suportada |
| `FalhaDaLupa::FormularioInesperado` | Formulário servido incompatível |
| `FalhaDaLupa::LinkAmbiguo` | Mais de um link assinado divergente |
| `FalhaDaLupa::LinkAusente` | Link não publicado |
| `FalhaDaLupa::LinkForaDaInstalacao` | Link fora da instalação autorizada |
| `ErroDasUnidades::LupaSemListaIntegral` | Inconsistência entre páginas e total |
| `FalhaDaLupa::ExcessoDePaginas` | Limite da coleta excedido |
| `ErroLupa::TabelaForaDaForma` | Página sem os campos observados |
