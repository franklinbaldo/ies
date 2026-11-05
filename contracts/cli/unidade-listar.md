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
`unidade_pedida_nao_verificavel_offline`: a CLI não finge ter
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
conferirem. O resultado avisa `origem: "sei"`, o número de
`requisicoes` e `persistida_no_indice` (false quando o índice
estiver indisponível). Sem índice, a leitura retorna sem persistir.

Erros tipados desta operação:
- `lupa_transporte_recusado`: falha de transporte ou status na navegação/consulta;
- `link_da_lupa_ausente`: ação obrigatória não foi publicada;
- `link_da_lupa_ambiguo`: links conflitantes para o mesmo destino;
- `link_da_lupa_fora_da_instalacao`: link fora da origem permitida;
- `charset_da_lupa_nao_suportado`: MIME não permite codificação segura;
- `formulario_da_lupa_inesperado`: controles/action da página não conferem;
- `pagina_da_lupa_fora_da_forma`: linha, tabela, cabeçalho ou paginação inválida;
- `lista_unidades_incompleta`: totais, páginas ou IDs não conferem;
- `lupa_excedeu_orcamento_de_paginas`: número de páginas não foi encerrado.

A lista local é um retrato da leitura, não uma garantia de atualidade.

## Tabela normativa de recusas da lupa

| Subcódigo | Condição |
| --- | --- |
| `charset_da_lupa_nao_suportado` | Codificação da tela não suportada |
| `formulario_da_lupa_inesperado` | Formulário servido incompatível |
| `link_da_lupa_ambiguo` | Mais de um link assinado divergente |
| `link_da_lupa_ausente` | Link não publicado |
| `link_da_lupa_fora_da_instalacao` | Link fora da instalação autorizada |
| `lista_unidades_incompleta` | Inconsistência entre páginas e total |
| `lupa_excedeu_orcamento_de_paginas` | Limite da coleta excedido |
| `pagina_da_lupa_fora_da_forma` | Página sem os campos observados |
