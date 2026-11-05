---
type: CommandContract
title: Prévia e salvamento seguro de documento
operation: documento.salvar
command: ies documento salvar DOC
implementation: partial
effect: write
---

# Prévia local e salvamento de documento

`ies documento salvar DOC [--versao N] [--confirmar]` compara, no
diretório atual, o par local de #207: `DOC.vN.html`, a **foto** que
`ies documento DOC --saida` gravou com o HTML servido da versão N
([versões e foto](documento-obter-versoes.md)), e `DOC.v(N+1).html`, a
edição que o agente cria a partir dela. Não há base nem manifesto
ocultos: a base é a própria foto. A decisão do proprietário de
05/11/2026 **substitui** o formato `DOC.vN.html` + `.DOC.vN.base.html` +
`.DOC.vN.meta.json` de #88.

## Escolha do par

- `--versao N`: a base é `DOC.vN.html` e a edição `DOC.v(N+1).html`.
- Sem `--versao`: a maior versão **local** M. Se `DOC.vM.html` é uma
  foto registrada com o mesmo SHA-256, a base é M (e a edição M + 1
  ainda falta); senão a edição é M e a base, M − 1. Isso não afirma a
  versão remota atual.

## Conferência da base

A foto só vale como base se o SHA-256 de `DOC.vN.html` é o que o
[índice local](../../docs/arquitetura/0003-indice-local.md) registrou
(tabela `foto`, por instalação, número SEI e versão) quando a leitura a
gravou ou a achou igual. A instalação vem da configuração de sessão
(`IES_ORIGEM_FILE`), lida só do disco. Sem configuração ou com o índice
desligado, nenhuma foto está registrada.

| Subcódigo | Fato |
| --- | --- |
| `base_versao_ausente` | nenhuma versão local, ou `DOC.vN.html` da base falta |
| `base_versao_nao_registrada` | o índice não tem o SHA-256 da foto vN |
| `base_versao_divergente` | a foto vN foi alterada depois de gravada |
| `versao_editada_ausente` | `DOC.v(N+1).html` falta |
| `versao_editada_igual_a_base` | `DOC.v(N+1).html` tem os mesmos bytes da foto |
| `versao_local_repetida`, `nome_versao_local_invalido` | duas grafias da mesma versão (`v2` e `v02`) ou nome fora da forma |
| `copia_local_link_simbolico`, `copia_local_permissoes_inseguras`, `destino_copia_inseguro`, `leitura_copia_falhou` | arquivo simbólico, aberto a grupo ou outros, não regular, acima de 8 MB, ou ilegível |

A prévia é **offline**, não expõe o corpo, URLs assinadas ou cookies e
não abre o editor. O JSON traz `previa` com `documento`, `versao_base`,
`versao_editada`, `bytes_base`, `bytes_editada`, `escrita_remota: false`
e `trecho_divergente`, e `confirmado: false`.

`--confirmar` **não envia POST no SEI**: retorna
`escrita_remota_nao_validada` após conferir o par. A escrita Web (#89)
enviará o maior `vN` local que o SEI ainda não tem, depois de conferir
ao vivo que a versão atual no SEI é a N; repetir depois de salvar não
fará nada. Mesclagem por seção, concorrência, assinatura e promoção
para uma nova versão dependem de validação por instalação.

- [Evidências legadas](../evidencias/documento-salvar.md)
- [Decisão arquitetural](../../docs/decisoes/documento-salvar.md)
- [Testes de CLI](../../tests/cli_documento_saida.rs)

## Localização conservadora das alterações (Refs #89)

`trecho_divergente` informa os intervalos de linhas **zero-based, fim
exclusivo** na base (`base_*`) e na edição (`trabalho_*`) que contêm
todas as alterações. Compara os prefixos e sufixos idênticos de linhas,
sem sessão, sem imprimir conteúdo e com memória constante. Duas ou mais
edições separadas podem produzir um único trecho com linhas inalteradas
entre elas: **não** é um diff mínimo nem prova equivalência com a versão
remota. A comparação é por bytes separados por LF, inclusive com HTML
não UTF-8, sem alterar arquivo algum.

## Leitura privada dos arquivos locais (Refs #89)

A prévia lê foto e edição como bytes. Em Linux, cada arquivo é aberto
com proteção contra link simbólico no componente final e contra
bloqueio em arquivo especial; a segurança (arquivo regular, limite 8 MB
e permissão sem acesso a grupo ou terceiros) é conferida no
**descritor já aberto**. Dados não UTF-8 permanecem bytes opacos. Os
controles não defendem contra a troca de diretórios ancestrais nem
contra alteração simultânea dos bytes depois da leitura.

## Inventário das versões locais (Refs #89, #207)

`ies documento salvar DOC --listar-copias [--json]` enumera **somente**
os `DOC.vN.html` do diretório atual, em ordem **numérica**, sem rede.
Não recebe `--versao` nem `--confirmar`; não equivale a listar versões
do SEI. Cada entrada traz `documento`, `versao`, `bytes` e `foto`
(`true`: SHA-256 igual ao registrado; `false`: arquivo local não
registrado, como a edição). Foto registrada e alterada recusa a lista
inteira (`foto_local_divergente`); também recusam diretório simbólico,
nome inválido, duas grafias da mesma versão e mais de 128 versões
(`limite_copias_locais`).

A saída JSON contém `copias`, `escopo: versoes_locais_verificadas`,
`versao_remota_confirmada: false` e `escrita_remota: false`. Lista
vazia significa só que não há arquivos desse número no diretório.
Nunca publicar conteúdo HTML nem alegar versão atual no servidor.

Live (05/11/2026, UnB 4.0.9): prévia **PASS** sobre a foto lida, com
`base_versao_divergente` depois de alterar um byte da foto
([ensaio](../../docs/validacao-live/2026-11-05-documento-saida-idempotente.md)).
