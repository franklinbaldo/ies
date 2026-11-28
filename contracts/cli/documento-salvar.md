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
  Se `N` for o maior `u32`, não existe `N+1`: recusar como
  `VersaoEditadaAusente`, sem reutilizar a própria base como edição.
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
  HTML servido e recusa com `ErroCopia::FotoDivergente`
  ([versões e foto](documento-obter-versoes.md));
- **salvar remoto (#89):** lê a versão atual no SEI e confere os bytes
  da foto local antes de enviar a edição.

| Erro (stderr) | Fato |
| --- | --- |
| `ErroCopia::BaseVersaoAusente` | nenhuma versão local, ou `DOC.vN.html` da base falta |
| `ErroCopia::VersaoEditadaAusente` | `DOC.v(N+1).html` falta |
| `ErroCopia::EdicaoIgualABase` | `DOC.v(N+1).html` tem os mesmos bytes da foto |
| `ErroCopia::VersaoLocalRepetida`, `ErroCopia::NomeVersaoLocalInvalido` | duas grafias da mesma versão (`v2` e `v02`) ou nome fora da forma |
| `ErroCopia::LinkSimbolico`, `ErroCopia::PermissoesInseguras`, `ErroCopia::DestinoInseguro`, `ErroCopia::LeituraFalhou` | arquivo simbólico, aberto a grupo ou outros, não regular, acima de 8 MB, ou ilegível |

A prévia não abre o editor nem executa escrita. **Na saída humana**, mostra
um recorte local com linhas da foto prefixadas por `-` e linhas da edição
por `+`, limitado a 1.800 caracteres por lado. É o envelope contínuo
entre prefixo e sufixo iguais, não um diff mínimo: linhas inalteradas
internas podem aparecer nos dois lados. Linhas não UTF-8 ou muito longas são omitidas, assim como
referências/URLs e atributos que carreguem valores ou sessão
(`href`, `src`, `action`, `value`, etc.). Atributos comuns de
apresentação (`class`, `style`, `id`) e texto com `=` ou `?`
continuam legíveis: não ocultar uma edição inteira de HTML comum.
O JSON continua sem conteúdo HTML, URLs assinadas ou cookies e traz
`previa` com `documento`, `versao_base`,
`versao_editada`, `bytes_base`, `bytes_editada`, `escrita_remota: false`,
`base_verificada: false` e `trecho_divergente`, e `confirmado: false`.

`--confirmar` está disponível nas instalações públicas de treinamento e
na bancada sintética `sei.test`. Na bancada, a CLI relê a versão N, confere
conteúdo idêntico à foto local, consulta o editor oferecido, deriva um
único POST do formulário servido e comprova a versão N+1 por GET
independente. Também relê o editor para observar a contagem de assinaturas
após a publicação; sem ambas as conferências, não declara sucesso.
O JSON confirmado informa `assinaturas_apos: 0` somente quando observado.
Resposta inconclusiva após a tentativa de POST resulta em
`ErroSalvarNoSei::EdicaoNaoComprovada` (saída 1), sem retry. O editor da simulação usa uma região
`main#corpoEditavel`.

Nas instalações de treinamento, edite somente o corpo principal da cópia
`DOC.v(N+1).html`. A CLI verifica que a minuta está sem assinatura,
confere a versão vigente e lê da tela quais seções o editor oferece.
O editor clássico usa o formulário publicado. O CKEditor 5 usa o JSON
da operação indicada pela tela. Em ambos, os valores das outras seções
vêm do editor. Se a foto e as seções não permitem localizar o corpo
com segurança, a CLI recusa sem gravar.

O fragmento editável permanece **HTML opaco**: nenhum parser HTML5
transforma tags, atributos, listas, tabelas ou marcação malformada do
corpo. A regra de LF canônico vale para **todo valor HTML incluído no POST**,
inclusive seções vizinhas obtidas do editor e não editadas pelo usuário:
CRCRLF, CRLF e CR isolado são convertidos em LF, como no Todos #1271/#1272
e na issue #856. Não há interpretação, mesclagem ou reescrita estrutural
do HTML de nenhuma seção. As seções vizinhas só passam por essa
canonicalização de quebras no momento de codificar o envio; os valores
servidos, os bytes da foto e a edição local permanecem intactos, e a
conferência da base anterior ao POST é literal. Não existe opção para
desativar LF canônico. A codificação URL-encoded do formulário clássico
ou a serialização JSON do CKEditor aplica-se *depois* da canonicalização.
Essa política também pode normalizar finais de linha de seções vizinhas
persistidas pelo SEI, efeito que deve ser incluído na revisão pré-merge
e na validação de treinamento, sem atribuir tal comportamento a testes
de escrita real ainda não realizados.

Após aceite da versão esperada, uma nova leitura confere a versão criada,
a identidade da seção principal e, pelo nome de cada seção enviada,
o **conteúdo efetivamente enviado** (HTML opaco com LF canônico).
Uma seção vazia deve estar presente; ausência não equivale a vazio.
A ordem das seções na releitura não altera essa comparação.
Aceite inconclusivo, falha pós-POST ou conteúdo divergente
resultam em `EdicaoNaoComprovada`, sem alegar qual versão foi persistida.
Não repetir a operação antes de conferência independente. Nenhuma tentativa
é repetida automaticamente.

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
(`ErroCopia::FotoDivergente`); a confirmação remota permanece bloqueada.
Aposentadoria de `base_versao_divergente`: #207, integrada pela PR #219.

## Produção (20/11/2026)

Esta escrita foi aprovada no ensaio das sete instalações de treinamento
([relatório](../../docs/validacao-live/2026-11-20-promocao-7-instalacoes.md))
e `--confirmar` grava também em produção. Esta seção prevalece sobre as
menções anteriores a "somente em treinamento". Prévia, POST único, releitura
e ausência de retry continuam obrigatórios.
