---
type: ArchitectureDecision
title: Gravação privada única de arquivos locais
description: Uma peça, crate::arquivo_privado, cria ou substitui todo arquivo privado do IES com a mesma política (nome final nunca seguido, diretório real não gravável por outros); cada módulo só traduz a falha tipada para os seus subcódigos
status: accepted
decision: peca-unica-temporario-mais-ligacao-ou-rename
---

# Gravação privada única de arquivos locais

Refs [#211](https://github.com/franklinbaldo/ies/issues/211) (e a
duplicata [#216](https://github.com/franklinbaldo/ies/issues/216), que
contou os módulos e pediu a ordem depois da #207). Revisada em
05/11/2026 por decisão do mantenedor: os ancestrais deixaram de ser
recusados por serem links ([modelo de ameaça](#modelo-de-ameaça)).

## Problema

Sete módulos repetiam `create_new(true)`, `0600`, `write_all`,
`sync_all` e limpeza, cada um com a sua política: `configuracao`
(sem `fsync` nem limpeza), `indice_local/abertura` (arquivo vazio e
diretório `0700`), `anexo_documento` e `foto_documento` (o gravador da
cópia depois da #207: parcial e `hard_link`), `arquivo_caixa` (só
`parent().is_dir()`), `arquivo_texto_documento` (confere links nos
ancestrais) e `sessao_login/gravacao` (temporário e `rename`). Não era
uma vulnerabilidade demonstrada; eram invariantes de segurança
espalhadas e divergentes.

## Alternativas

- **(a) manter as sete cópias** e alinhar a política à mão. Cada nova
  gravação copiaria a mais próxima; a divergência voltaria. Rejeitada.
- **(b) peça dentro de `transporte`**, ao lado da leitura
  `transporte::arquivo_privado`. A leitura está lá porque é dos arquivos
  de sessão do transporte; a escrita serve a exportação da caixa, a
  configuração, o índice e o `--saida`, nada disso transporte, e
  `transporte.rs` já passa de 12 KB. Rejeitada.
- **(c) `src/arquivo_privado.rs`** (escolhida): o módulo
  `crate::arquivo_privado` é o par de escrita da leitura
  `transporte::arquivo_privado` e não conhece nenhum formato, nome ou
  subcódigo. Os módulos continuam donos da serialização, dos nomes e dos
  erros públicos.
- **(d) arquivo temporário anônimo** (`O_TMPFILE` + `linkat`): nenhum
  parcial apareceria nem depois de um `kill -9`. Exige `libc` (a `std` não
  liga `linkat` com `AT_SYMLINK_FOLLOW` nem abre `O_TMPFILE` com nome
  vinculável), só existe no Linux e nem todo sistema de arquivos o
  aceita. Rejeitada; ver parciais órfãos abaixo.

## Política da peça

O chamador declara o modo em `Politica`:

| Garantia | `exclusiva()` | `substituicao()` |
|---|---|---|
| Destino existente (inclusive link) | `DestinoJaExiste`, nada muda | arquivo regular é trocado inteiro; link ou não regular é `DestinoNaoEhArquivoRegular` |
| Escrita | temporário `0600` no mesmo diretório, `sync_all` | igual |
| Publicação | `hard_link` para o nome final (falha se o nome surgiu) | `rename` sobre o nome final |
| Concorrência | dois criadores: um vence, o outro recebe `DestinoJaExiste` | o último `rename` vence; o lock é do chamador (sessão) |

Em ambos os modos:

- **Nome final.** Nunca seguido: `create_new` e `hard_link` falham se o
  nome existe (inclusive link pendente), e `rename` substitui o link em
  vez de escrever através dele.
- **Diretório do destino.** O pai do destino (absoluto pelo diretório
  corrente) tem de existir e ser diretório, senão `DiretorioAusente`;
  links no caminho, inclusive o próprio `DIR` dado pela pessoa, são
  aceitos. O pai é resolvido com `fs::canonicalize` e o **diretório
  real** é examinado e usado para o temporário e o nome final: no Unix,
  gravável por grupo ou outros (`mode & 0o022`) sem sticky bit
  (`0o1000`) é `DiretorioGravavelPorOutros`. Fora do Unix não há modo e
  a conferência não se aplica. Com `criando_diretorios()` (só o índice)
  cada nível ausente nasce `0700`, um por vez.
- **Limite.** `com_limite(n)` recusa com `AcimaDoLimite` antes de passar
  de `n` bytes.
- **Falha no meio.** Qualquer falha antes da publicação remove o
  temporário (no `Drop` do `Parcial`); o nome final nunca existe pela
  metade, nem no modo exclusivo, que antes escrevia direto no destino.
- **Diretório.** Depois da publicação, `sync_all` do diretório (melhor
  esforço, como já fazia a sessão).
- **Fora do Unix** não há modo de arquivo: vale a ACL do perfil, como na
  leitura; `cfg(unix)` só no modo e na conferência do diretório. A
  publicação é a mesma.

### Modelo de ameaça

A ameaça é a clássica do link plantado (*symlink/clobber*): outro
usuário que pode escrever no diretório do destino planta, no nome que o
IES vai usar, um link para um arquivo da vítima (ou para um nome que ele
vai ler depois) e espera o IES escrever através dele. Duas regras
fecham isso:

1. o nome final nunca é seguido, em nenhum modo (acima);
2. o diretório real do destino não pode ser gravável por mais ninguém,
   exceto com sticky bit (`/tmp`), onde outro usuário não remove nem
   troca o que é nosso e a criação exclusiva não segue o que ele
   plantou.

Ancestrais não são ameaça nesse modelo: quem pode trocar um ancestral
por link já pode escrever no diretório que o contém, e o diretório real
é conferido depois de resolvido. Os arquivos da sessão (cookie, URL,
origem e o marcador `.geracao`) continuam lidos por
`transporte::arquivo_privado`, que recusa link no nome final; as outras
leituras não mudaram.

### Alternativa rejeitada: recusar ancestral link (#211, até 05/11/2026)

A primeira versão desta peça lia cada ancestral, da raiz ao pai, com
`symlink_metadata` e recusava qualquer link
(`DiretorioLinkSimbolico`). Rejeitada pelo mantenedor em 05/11/2026:

- quebra instalações legítimas: no macOS `/var`, `/tmp` e `/etc` são
  links para `/private/...` e o `$TMPDIR` fica sob `/var/folders`; no
  Fedora Silverblue/Kinoite `/home` é link para `/var/home`. Nesses
  sistemas o `ies login` faria todas as requisições e falharia ao
  gravar a sessão;
- não fecha a corrida que dizia fechar: a conferência era por caminho,
  e um diretório pode ser trocado por link entre ela e a criação (seria
  preciso `openat` por componente, fora da `std`); esta decisão já
  registrava esse limite;
- não protege contra a ameaça real, que é o nome final e o diretório
  gravável por outros.

### O que a peça não garante

A conferência do diretório é por caminho: um processo do **mesmo
usuário**, ou de quem pode escrever num ancestral do diretório real,
que troque o diretório entre a conferência e a criação não é impedido.
Só o diretório imediato é conferido, não os ancestrais dele. Um `DIR`
criado com `umask 002` (grupo privado do usuário) é gravável pelo grupo
e é recusado: use `chmod g-w DIR`.

### Parciais órfãos (o resto da #207)

O temporário chama-se `.NOME.parcial-PID-N` (N é um contador do
processo) no diretório do destino. Um `kill -9` ou queda de energia
entre a criação e a publicação não executa código nenhum, então o
parcial pode sobrar: isso não tem como ser evitado só com a `std`
(alternativa d). A peça reduz o dano: a próxima gravação do **mesmo**
nome remove os `.NOME.parcial-*` regulares com mais de uma hora sem
modificação (um parcial vivo é escrito ou publicado em segundos). Os
nomes antigos da #207 (`.DOC.vN.html.parcial-PID`) caem na mesma regra.
Parciais de outro nome nunca são tocados.

## Migração

| Módulo | Modo | Subcódigos mantidos |
|---|---|---|
| `configuracao::salvar_arquivos_de_acesso` | exclusiva | `configuracao_ja_existe_ou_sem_permissao` (destino, diretório, criação), `configuracao_nao_gravada` (escrita, publicação) |
| `indice_local/abertura` | exclusiva, criando diretórios | nenhum público: qualquer falha desliga o índice; destino já existente é o banco de outro `ies` |
| `anexo_documento` | `Parcial` exclusivo, publicado depois | `diretorio_do_anexo_ausente`, `diretorio_do_anexo_gravavel_por_outros`, `arquivo_do_anexo_ja_existe`, `escrita_do_anexo_falhou` |
| `arquivo_caixa` | exclusiva | os de antes e `diretorio_do_arquivo_da_caixa_gravavel_por_outros` |
| `sessao_login/gravacao` | `Parcial` de substituição, um por arquivo, publicados em ordem | `destino_da_sessao_nao_eh_arquivo_regular` (só o nome final), `diretorio_da_sessao_gravavel_por_outros`, `gravacao_da_sessao_falhou` |
| `foto_documento` (cópia da #207) | exclusiva | `foto_local_divergente` (o nome surgiu), `gravacao_foto_falhou`; o `DIR` como no texto |
| `arquivo_texto_documento` | exclusiva | os de antes, com `diretorio_texto_documento_gravavel_por_outros` no lugar do de link |

### Subcódigos por módulo, não um compartilhado

O diretório gravável por outros ganhou um subcódigo **por módulo**, na
família que cada módulo já tinha (`diretorio_do_anexo_*`,
`diretorio_do_arquivo_da_caixa_*`, `diretorio_texto_documento_*`,
`diretorio_da_sessao_*`), no lugar dos de link que deixaram de existir.
Um código compartilhado seria mais curto, mas quebraria a regra desta
decisão (cada módulo traduz a falha da peça para o seu vocabulário), e
a mensagem de cada um nomeia a opção que a pessoa corrige (`--saida`,
o diretório da sessão). A configuração continua com
`configuracao_ja_existe_ou_sem_permissao` (o nome já cobre o caso) e o
índice continua sem subcódigo público (desliga).

O lock do login e o lock lateral do índice abrem com `create(true)` sem
conteúdo e ficam fora: não são gravações de dados. Nenhum comando ganhou
requisição.

## Verificação

- Testes da peça em `src/arquivo_privado/tests.rs`: arquivo novo `0600`,
  destino existente, link no destino (inclusive pendente e plantado
  depois de abrir o parcial), ancestral link aceito (`/home` →
  `/var/home` simulado), `DIR` link aceito, diretório gravável por
  grupo ou outros recusado, sticky aceito, diretório ausente,
  diretórios `0700`, limite, falha no meio sem parcial, nome que surge
  antes da publicação, oito criadores exclusivos com um só vencedor,
  varredura só de parciais antigos do mesmo nome e
  `nenhuma_outra_gravacao_privada_usa_create_new` (o `grep` como teste).
- Por módulo: caixa, configuração, índice, sessão, texto, anexo e foto
  testam o diretório por link aceito e o gravável por outros recusado
  com o subcódigo do módulo; o retrato da sessão (#222) testa a recusa
  de link no nome final de cada arquivo.

## Revisitar quando

- a `std` expuser `openat`/`linkat` com flags: conferir ancestrais por
  descritor e usar temporário anônimo;
- um sistema de arquivos suportado não tiver `hard_link` (o modo
  exclusivo falharia com `ConfirmacaoFalhou`).
