---
type: Guide
title: Erros locais
description: Configuração local e gravação privada de arquivos
---

# Erros locais

Parte do [guia de erros tipados](../erros-tipados.md), separada por responsabilidade.

## Configuração local: causas tipadas (#212)

`ler_configuracao_local` devolve `Result`, com causas internas
separadas para ausência, metadados, link simbólico, não regular, tamanho
excessivo, permissão aberta, leitura, JSON e campos inválidos. Não inclui
caminhos nem conteúdo no erro ou na saída pública.

O subcódigo legado `arquivo_configuracao_nao_pode_ser_lido` permanece
para arquivo ausente, não regular, excessivo ou ilegível. Novas causas
com remediação própria: `arquivo_configuracao_link_simbolico`,
`arquivo_configuracao_permissoes_inseguras`,
`arquivo_configuracao_json_invalido` e
`arquivo_configuracao_campo_invalido`. O campo inválido inclui
caminho ausente, não textual, vazio ou relativo. Não há fallback
para variáveis individuais quando `IES_CONFIG_FILE` é informado.
A verificação usa `symlink_metadata` antes da abertura e confere
o arquivo aberto, sem consumir ou registrar dados de sessão.

## Gravação privada local (#211)

Toda gravação de arquivo privado passa por `crate::arquivo_privado`,
que devolve uma falha tipada sem caminho nem conteúdo (destino já
existente, destino não regular, diretório ausente, diretório gravável
por outros, limite, criação, escrita, publicação). Cada módulo a traduz
para os seus subcódigos
([decisão](../decisoes/arquivos-privados/gravacao-privada.md#migração)).

Desde 05/11/2026 link simbólico em ancestral, ou no próprio `DIR` dado
pela pessoa, é **aceito**: o diretório é resolvido com `canonicalize`.
O modo do diretório real só é conferido nos caminhos que o IES gerencia
(sessão, índice, configuração): no Unix, gravável por grupo ou outros
sem sticky bit é recusado. Nos diretórios que a pessoa escolhe
(`--saida DIR`, o arquivo da caixa) o modo é dela e não é conferido
([modelo de ameaça](../decisoes/arquivos-privados/gravacao-privada.md#modelo-de-ameaça)).
Mudanças públicas:

| Subcódigo | Mudança |
|---|---|
| `diretorio_do_arquivo_da_caixa_link_simbolico` | removido (novo na PR #232, não chegou ao `main`) |
| `diretorio_do_anexo_link_simbolico` | removido; o `DIR` por link é aceito |
| `diretorio_texto_documento_link_simbolico` | removido (texto, foto e `--saida` dos comandos de documento); o `DIR` por link é aceito |
| `diretorio_da_sessao_gravavel_por_outros` | novo: diretório real da sessão gravável por grupo ou outros sem sticky bit, no `ies login` e na troca de unidade |
| `destino_da_sessao_nao_eh_arquivo_regular` | muda: só o nome final (link ou não regular), não mais ancestral link |
| `configuracao_ja_existe_ou_sem_permissao` | muda: cobre o diretório gravável por outros sem sticky bit, não mais o ancestral link |
| `destino_copia_inseguro` | muda: na gravação das fotos, o `DIR` por link deixa de ser recusado (não diretório continua) |

O índice local não tem subcódigo público: diretório gravável por outros
o desliga, como antes. Nenhum subcódigo novo para `--saida` ou caixa.

O nome final nunca é seguido em nenhum modo, e os arquivos da sessão
continuam recusados como link no nome final
(`ErroArquivoPrivado::LinkSimbolico`).
