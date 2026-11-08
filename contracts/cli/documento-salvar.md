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
- Sem `--versao`: a maior versão **local** M. Se `DOC.v(M−1).html`
  existe, a base é M − 1 e a edição M; senão a base é M e a edição
  M + 1 ainda falta. Só os nomes contam; isso não afirma a versão
  remota atual.

## Base não verificada

A prévia é **puramente offline**: não lê o índice local, a configuração
nem os arquivos de sessão, e por isso **não verifica** que `DOC.vN.html`
ainda é o que o SEI serviu. Diz isso na saída: `base_verificada: false`
no JSON e "base não verificada" no texto. A base é conferida com o SEI
em dois outros pontos:

- **releitura:** `ies documento DOC --saida` compara a foto local com o
  HTML servido e recusa com `foto_local_divergente`
  ([versões e foto](documento-obter-versoes.md));
- **salvar remoto (#89, futuro):** lerá a versão N do SEI para confirmar
  que ainda é a atual e comparará o hash com a vN local.

| Subcódigo | Fato |
| --- | --- |
| `base_versao_ausente` | nenhuma versão local, ou `DOC.vN.html` da base falta |
| `versao_editada_ausente` | `DOC.v(N+1).html` falta |
| `versao_editada_igual_a_base` | `DOC.v(N+1).html` tem os mesmos bytes da foto |
| `versao_local_repetida`, `nome_versao_local_invalido` | duas grafias da mesma versão (`v2` e `v02`) ou nome fora da forma |
| `copia_local_link_simbolico`, `copia_local_permissoes_inseguras`, `destino_copia_inseguro`, `leitura_copia_falhou` | arquivo simbólico, aberto a grupo ou outros, não regular, acima de 8 MB, ou ilegível |

A prévia não abre o editor nem executa escrita. **Na saída humana**, mostra
um recorte local com linhas da foto prefixadas por `-` e linhas da edição
por `+`, limitado a 1.800 caracteres por lado. É o envelope contínuo
entre prefixo e sufixo iguais, não um diff mínimo: linhas inalteradas
internas podem aparecer nos dois lados. Linhas não UTF-8, muito longas
ou com atributos/referências potencialmente sensíveis são omitidas.
O JSON continua sem conteúdo HTML, URLs assinadas ou cookies e traz
`previa` com `documento`, `versao_base`,
`versao_editada`, `bytes_base`, `bytes_editada`, `escrita_remota: false`,
`base_verificada: false` e `trecho_divergente`, e `confirmado: false`.

`--confirmar` **continua bloqueado para instalações reais do SEI**:
retorna `escrita_remota_nao_validada` após conferir o par.
Somente no host sintético `sei.test`, com sessão de ensaio, a CLI
executa os BDD-89-02/03/04 sintéticos: relê a versão N, confere
conteúdo idêntico à foto local, consulta o editor oferecido, deriva um
único POST do formulário servido e comprova a versão N+1 por GET
independente. Também relê o editor para observar a contagem de assinaturas
após a publicação; sem ambas as conferências, não declara sucesso.
O JSON confirmado informa `assinaturas_apos: 0` somente quando observado.
Resposta inconclusiva após a tentativa de POST resulta em
`efeito_incerto`, sem retry. O editor da simulação usa uma região
`main#corpoEditavel`; essa hipótese **não representa uma compatibilidade
validada com o HTML de editores reais do SEI**. Mesclagem por seção e
validação em treinamento continuam fora do escopo.

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
do SEI. Cada entrada traz `documento`, `versao` e `bytes`; não diz qual
arquivo é foto e qual é edição, porque offline não há como saber. Um
arquivo inseguro recusa a lista inteira, assim como diretório simbólico,
nome inválido, duas grafias da mesma versão e mais de 128 versões
(`limite_copias_locais`).

A saída JSON contém `copias`, `escopo: arquivos_locais`,
`versao_remota_confirmada: false` e `escrita_remota: false`. Lista
vazia significa só que não há arquivos desse número no diretório.
Nunca publicar conteúdo HTML nem alegar versão atual no servidor.

O escopo `arquivos_locais` diz o que a lista é: arquivos `DOC.vN.html`
conferidos só quanto à segurança da leitura (regular, privado, nome na
forma), sem conteúdo verificado com o SEI ou com registro algum; o
antigo `versoes_locais_verificadas` sugeria uma verificação que não há.

Live (05/11/2026, UnB 4.0.9): prévia **PASS** sobre a foto lida
([ensaio](../../docs/validacao-live/2026-11-05-documento-saida-idempotente.md)).
Naquele ensaio a foto v3 alterada em um byte deu `base_versao_divergente`;
essa conferência saiu da prévia e passou à releitura
(`foto_local_divergente`); a confirmação remota permanece bloqueada.
Aposentadoria de `base_versao_divergente`: #207, integrada pela PR #219.
